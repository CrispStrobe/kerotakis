/* Actual chroot/seccomp controls. Contains no application credentials. */
#define _GNU_SOURCE
#include <stdio.h>
#include <unistd.h>
#include <stdlib.h>
#include <fcntl.h>
#include <sys/syscall.h>
#include <sys/socket.h>
#include <signal.h>
#include <string.h>
#include <sys/resource.h>
int main(void) {
 int readonly=open("/input.lab",O_RDONLY); char before[32]={0}; if(readonly>=0) read(readonly,before,31);
 int a=syscall(SYS_open,"/input.lab",O_RDONLY|O_TRUNC,0);
 int b=syscall(SYS_openat,AT_FDCWD,"/input.lab",O_RDONLY|O_TRUNC,0);
 int writefd=open("/input.lab",O_WRONLY);
 int created=open("/created",O_CREAT|O_RDWR,0600);
 int owner=fcntl(1,F_SETOWN,getppid());
 int ownerex=fcntl(1,F_SETOWN_EX,&(struct f_owner_ex){.type=F_OWNER_PID,.pid=getppid()});
 int sig=fcntl(1,F_SETSIG,SIGIO);
 int async=fcntl(1,F_SETFL,O_ASYNC);
 FILE *host=fopen("/etc/passwd","r");
 char after[32]={0}; int reader=open("/input.lab",O_RDONLY); if(reader>=0) read(reader,after,31);
 struct rlimit cpu,mem; int bounded=getrlimit(RLIMIT_CPU,&cpu)==0 && getrlimit(RLIMIT_AS,&mem)==0
  && cpu.rlim_cur==12 && cpu.rlim_max==12 && mem.rlim_cur==1073741824 && mem.rlim_max==1073741824;
 int all=bounded && readonly>=0 && a<0 && b<0 && writefd<0 && created<0 && owner<0 && ownerex<0 && sig<0 && async<0 && !host
  && !strcmp(before,after) && !getenv("KERO_API_KEYS") && !getenv("KV_REST_API_TOKEN") && getuid()==65534 && getgid()==65534
  && socket(AF_INET,SOCK_STREAM,0)<0 && fork()<0 && kill(getppid(),0)<0 && setuid(0)<0 && chroot("/")<0;
 printf("{\"isolation_controls_pass\":%s}\n",all?"true":"false");
 return all?0:1;
}
