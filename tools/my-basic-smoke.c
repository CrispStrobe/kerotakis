/* Project-owned smoke checks for the pinned MIT MY-BASIC core. */
#include "my_basic.h"

#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static char printed[256];
static size_t printed_len;
static int error_count;

static void fail(const char* message) {
	fprintf(stderr, "MY-BASIC smoke failure: %s\n", message);
	exit(EXIT_FAILURE);
}

static void require(int condition, const char* message) {
	if(!condition)
		fail(message);
}

static int capture_print(struct mb_interpreter_t* interpreter, const char* format, ...) {
	va_list args;
	int written;
	(void)interpreter;

	if(printed_len >= sizeof(printed))
		return 0;

	va_start(args, format);
	written = vsnprintf(
		printed + printed_len,
		sizeof(printed) - printed_len,
		format,
		args
	);
	va_end(args);
	if(written > 0) {
		size_t available = sizeof(printed) - printed_len;
		printed_len += (size_t)written < available ? (size_t)written : available - 1;
	}

	return written;
}

static int reject_input(
	struct mb_interpreter_t* interpreter,
	const char* prompt,
	char* buffer,
	int length
) {
	(void)interpreter;
	(void)prompt;
	if(buffer && length > 0)
		buffer[0] = '\0';
	return 0;
}

static int reject_import(struct mb_interpreter_t* interpreter, const char* path) {
	(void)interpreter;
	(void)path;
	return MB_FUNC_ERR;
}

static void capture_error(
	struct mb_interpreter_t* interpreter,
	mb_error_e error,
	const char* description,
	const char* file,
	int position,
	unsigned short row,
	unsigned short column,
	int abort_code
) {
	(void)interpreter;
	(void)file;
	(void)position;
	(void)abort_code;
	(void)error;
	(void)description;
	(void)row;
	(void)column;
	++error_count;
}

static int native_add(struct mb_interpreter_t* interpreter, void** local) {
	int_t left = 0;
	int_t right = 0;

	mb_check(mb_attempt_open_bracket(interpreter, local));
	mb_check(mb_pop_int(interpreter, local, &left));
	mb_check(mb_pop_int(interpreter, local, &right));
	mb_check(mb_attempt_close_bracket(interpreter, local));
	mb_check(mb_push_int(interpreter, local, left + right));

	return MB_FUNC_OK;
}

static int suspend_now(struct mb_interpreter_t* interpreter, void** local) {
	mb_check(mb_attempt_open_bracket(interpreter, local));
	mb_check(mb_attempt_close_bracket(interpreter, local));
	mb_check(mb_push_int(interpreter, local, 1));
	mb_check(mb_schedule_suspend(interpreter, MB_FUNC_SUSPEND));

	return MB_FUNC_OK;
}

static struct mb_interpreter_t* new_interpreter(void) {
	struct mb_interpreter_t* interpreter = NULL;
	require(mb_open(&interpreter) == MB_FUNC_OK, "mb_open");
	require(interpreter != NULL, "mb_open returned null");
	require(mb_set_printer(interpreter, capture_print) == MB_FUNC_OK, "set printer");
	require(mb_set_inputer(interpreter, reject_input) == MB_FUNC_OK, "set input callback");
	require(mb_set_import_handler(interpreter, reject_import) == MB_FUNC_OK, "set import callback");
	require(mb_set_error_handler(interpreter, capture_error) == MB_FUNC_OK, "set error callback");
	return interpreter;
}

static void close_interpreter(struct mb_interpreter_t** interpreter) {
	require(mb_close(interpreter) == MB_FUNC_OK, "mb_close");
	require(*interpreter == NULL, "mb_close did not clear pointer");
}

static void check_value_int(struct mb_interpreter_t* interpreter, const char* name, int_t expected) {
	mb_value_t value;
	require(mb_get_value_by_name(interpreter, NULL, name, &value) == MB_FUNC_OK, "get integer");
	require(value.type == MB_DT_INT, "integer type");
	require(value.value.integer == expected, "integer value");
}

static void check_language_and_callback(void) {
	static const char program[] =
		"begin:\n"
		"DIM VALUES(3)\n"
		"FOR I = 0 TO 2\n"
		"  VALUES(I) = I * I\n"
		"NEXT I\n"
		"TOTAL = VALUES(0) + VALUES(1) + VALUES(2)\n"
		"TEXT$ = \"KERO\" + \"TAKIS\"\n"
		"CALLBACK_RESULT = NATIVE_ADD(20, 22)\n"
		"IF TOTAL = 5 THEN STATUS = 1 ELSE STATUS = 0\n"
		"PRINT TEXT$\n";
	struct mb_interpreter_t* interpreter = new_interpreter();
	mb_value_t value;

	printed[0] = '\0';
	printed_len = 0;
	require(mb_register_func(interpreter, "NATIVE_ADD", native_add) != 0, "register callback");
	require(mb_load_string(interpreter, program, true) == MB_FUNC_OK, "load language program");
	require(mb_run(interpreter, false) == MB_FUNC_OK, "run language program");
	check_value_int(interpreter, "TOTAL", 5);
	check_value_int(interpreter, "CALLBACK_RESULT", 42);
	check_value_int(interpreter, "STATUS", 1);
	require(mb_get_value_by_name(interpreter, NULL, "TEXT$", &value) == MB_FUNC_OK, "get string");
	require(value.type == MB_DT_STRING, "string type");
	require(strcmp(value.value.string, "KEROTAKIS") == 0, "string value");
	require(strstr(printed, "KEROTAKIS") != NULL, "captured print output");
	close_interpreter(&interpreter);
}

static void check_errors_and_file_loading(void) {
	struct mb_interpreter_t* interpreter = new_interpreter();
	int before = error_count;

	require(mb_load_string(interpreter, "IF THEN\n", true) == MB_FUNC_OK, "load invalid program fixture");
	require(mb_run(interpreter, true) != MB_FUNC_OK, "invalid syntax accepted");
	require(error_count > before, "syntax error callback not invoked");
	require(mb_load_file(interpreter, "must-not-be-opened.bas") != MB_FUNC_OK, "file loading enabled");
	close_interpreter(&interpreter);
}

static void check_cancellation(void) {
	static const char program[] =
		"BEFORE = 1\n"
		"CANCELLED = SUSPEND_NOW()\n"
		"AFTER = 1\n";
	struct mb_interpreter_t* interpreter = new_interpreter();
	mb_value_t value;

	require(mb_register_func(interpreter, "SUSPEND_NOW", suspend_now) != 0, "register suspend callback");
	require(mb_load_string(interpreter, program, true) == MB_FUNC_OK, "load cancellation program");
	require(mb_run(interpreter, false) == MB_FUNC_SUSPEND, "execution did not suspend");
	check_value_int(interpreter, "BEFORE", 1);
	require(mb_get_value_by_name(interpreter, NULL, "AFTER", &value) == MB_FUNC_OK, "get post-suspend value");
	require(
		value.type == MB_DT_NIL || (value.type == MB_DT_INT && value.value.integer != 1),
		"execution continued after suspension"
	);
	close_interpreter(&interpreter);
}

int main(void) {
	require(mb_init() == MB_FUNC_OK, "mb_init");
	check_language_and_callback();
	check_errors_and_file_loading();
	check_cancellation();
	require(mb_dispose() == MB_FUNC_OK, "mb_dispose");
	return EXIT_SUCCESS;
}
