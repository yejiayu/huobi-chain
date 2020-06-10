#include <pvm.h>
#include <pvm_extend.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>

#define ERROR_METHOD_NOT_FOUND 1000
#define ERROR_GET_ADDRESS 1001

uint64_t get_address(char *address) {
    char args[100] = {0};

    uint64_t args_len = pvm_load_args(args);
    if (args_len < 42) {
        return ERROR_GET_ADDRESS;
    }

    for (int i = 1; i <= 42; i++) {
        address[i - 1] = args[i];
    }
    return 0;
}

uint64_t do_write(char *contract_address, char *args) {
    const char* service = "riscv";
    const char* method = "exec";

    char payload[1024];
    sprintf(payload, "{\"address\": \"%s\", \"args\": \"%s\"}", contract_address, args);

    uint8_t ret[1024] = {0};
    uint64_t ret_len = pvm_service_write(service, method, payload, strlen(payload), ret);

    pvm_ret(ret, ret_len);
    return 0;
}

uint64_t do_read(char *contract_address, char *args) {
    const char* service = "riscv";
    const char* method = "call";

    char payload[1024];
    sprintf(payload, "{\"address\": \"%s\", \"args\": \"%s\"}", contract_address, args);

    uint8_t ret[1024] = {0};
    uint64_t ret_len = pvm_service_read(service, method, payload, strlen(payload), ret);

    pvm_ret(ret, ret_len);
    return 0;
}

uint64_t write(char *target, bool append) {
    char contract_address[41] = {0};

    uint64_t ret_code = get_address(contract_address);
    if (ret_code != 0) {
        return ret_code;
    }

    char args[100] = {0};
    if (append == true) {
        sprintf(args, "%s%s", target, contract_address);
    } else {
        sprintf(args, "%s", target);
    }

    return do_write(contract_address, args);
}

uint64_t read(char *target, bool append) {
    char contract_address[41] = {0};

    uint64_t ret_code = get_address(contract_address);
    if (ret_code != 0) {
        return ret_code;
    }

    char args[100] = {0};
    if (append == true) {
        sprintf(args, "%s%s", target, contract_address);
    } else {
        sprintf(args, "%s", target);
    }

    return do_read(contract_address, args);
}

uint64_t msg() {
    pvm_ret("1vz411b7WB", strlen("1vz411b7WB"));
    return 0;
}

uint64_t r() {
    char msg[100] = {0};
    uint64_t msg_len = pvm_get_storage("crpd", strlen("crpd"), msg);
    pvm_ret(msg, msg_len);
    return 0;
}

uint64_t w() {
    pvm_set_storage("crpd", strlen("crpd"), "1vz411b7WB", strlen("1vz411b7WB"));
    return 0;
}

/*
 *          write                write
 * c() ---------------> b() ---------------> w()
 */
uint64_t b() {
    return write("w", false);
}


uint64_t c() {
    return write("b", true);
}

/*
 *          read                 write
 * f() ---------------> e() ---------------> w()
 */

uint64_t e() {
    return write("w", false);
}
uint64_t f() {
    return read("e", true);
}

/*
 *          read                  read
 * y() ---------------> x() ---------------> r()
 */
uint64_t x() {
    return read("r", false);
}

uint64_t y() {
    return read("x", true);
}

int main() {
    char args[1024] = {0};
    uint64_t args_len = pvm_load_args(args);

    if (args_len == 0) {
        pvm_ret_str("method not found");
        return ERROR_METHOD_NOT_FOUND;
    }

    uint64_t ret = 0;
    char method = args[0];

    if (method == 'r') {
        ret = r();
    } else if (method == 'w') {
        ret = w();
    } else if (method == 'b') {
        ret = b();
    } else if (method == 'c') {
        ret = c();
    } else if (method == 'e') {
        ret = e();
    } else if (method == 'f') {
        ret = f();
    } else if (method == 'x') {
        ret = x();
    } else if (method == 'y') {
        ret = y();
    } else if (method == 'm') {
        ret = msg();
    } else {
        pvm_ret_str("method not found");
        return ERROR_METHOD_NOT_FOUND;
    }

    return ret;
}
