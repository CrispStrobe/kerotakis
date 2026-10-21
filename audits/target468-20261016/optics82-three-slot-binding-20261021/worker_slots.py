"""Pure three-slot renderer/independent normalizer; never executes a worker."""
import ast,hashlib,json
SLOTS={'CONFIG','CONFIG_SHA256','EXPECTED_RELEASE'}
NON_BINDING_RUNTIME_NULLS={'WORK_DEADLINE'} # Actual frozen worker's runtime clock; never replaced or normalized.
MAX_WORKER_BYTES=1024**2

def canonical(value):return json.dumps(value,sort_keys=True,separators=(',',':'),allow_nan=False).encode()
def require(value,message):
 if not value:raise ValueError(message)
def normalized(raw):
 tree=ast.parse(raw);nodes={};values={}
 for node in tree.body:
  if isinstance(node,ast.Assign)and len(node.targets)==1 and isinstance(node.targets[0],ast.Name):
   name=node.targets[0].id
   if name in SLOTS:
    require(name not in nodes,'duplicate worker slot: '+name);nodes[name]=node;values[name]=ast.literal_eval(node.value)
   elif isinstance(node.value,ast.Constant)and node.value.value is None and name not in NON_BINDING_RUNTIME_NULLS:
    raise ValueError('unknown null worker slot: '+name)
 stores=[node.id for node in ast.walk(tree)if isinstance(node,ast.Name)and isinstance(node.ctx,ast.Store)and node.id in SLOTS]
 require(set(nodes)==SLOTS and len(stores)==3 and set(stores)==SLOTS,'missing, non-simple or shadowed worker slot')
 for node in nodes.values():node.value=ast.Constant(None)
 return ast.dump(tree,include_attributes=False),values

def validated_values(config,config_sha,expected_release):
 require(type(config)is dict and config and type(expected_release)is dict and expected_release,'unbound CONFIG or EXPECTED_RELEASE')
 require(type(config_sha)is str and hashlib.sha256(canonical(config)).hexdigest()==config_sha,'CONFIG_SHA256 does not match canonical actual CONFIG')
 require(config['runtime_source']==expected_release['source']and config['runtime_tree']==expected_release['tree'],'release/source/tree differs')
 canonical(expected_release) # Reject nonfinite/non-JSON values before serialization.
 return {'CONFIG':config,'CONFIG_SHA256':config_sha,'EXPECTED_RELEASE':expected_release}

def check_bound_source(template_raw,bound_raw,config,config_sha,expected_release):
 expected=validated_values(config,config_sha,expected_release)
 template_ast,template_values=normalized(template_raw);bound_ast,bound_values=normalized(bound_raw)
 require(template_values==dict.fromkeys(SLOTS),'template is already bound')
 require(bound_values==expected,'bound slots differ from independently validated actual inputs')
 require(template_ast==bound_ast,'worker changed outside exact three slots')
 require(len(bound_raw)<MAX_WORKER_BYTES,'final serialized worker is not below1MiB')

def render_worker(template_raw,config,config_sha,expected_release):
 require(type(template_raw)is bytes,'worker bytes required')
 values=validated_values(config,config_sha,expected_release)
 _,old=normalized(template_raw);require(old==dict.fromkeys(SLOTS),'template is already bound')
 tree=ast.parse(template_raw);lines=template_raw.splitlines(keepends=True);offsets=[0]
 for line in lines:offsets.append(offsets[-1]+len(line))
 replacements=[]
 for node in tree.body:
  if isinstance(node,ast.Assign)and len(node.targets)==1 and isinstance(node.targets[0],ast.Name)and node.targets[0].id in SLOTS:
   name=node.targets[0].id;replacements.append((offsets[node.lineno-1]+node.col_offset,offsets[node.end_lineno-1]+node.end_col_offset,(name+'='+repr(values[name])).encode()))
 bound=template_raw
 for begin,end,value in sorted(replacements,reverse=True):bound=bound[:begin]+value+bound[end:]
 check_bound_source(template_raw,bound,config,config_sha,expected_release)
 return bound
