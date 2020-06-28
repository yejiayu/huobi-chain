#include <pvm.h>
#include <pvm_extend.h>

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

uint64_t a() {
    pvm_assert(1 > 2, "1 should never bigger than 2");
}

uint64_t b(char *contract_address) {
    const char* service = "riscv";
    const char* method = "call";

    char payload[1024];
    sprintf(payload, "{\"address\": \"%s\", \"args\": \"a\"}", contract_address);

    uint8_t ret[1024] = {0};
    uint64_t ret_len = pvm_service_read(service, method, payload, strlen(payload), ret);
    return 0;
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

    if (method == 'a') {
        ret = a();
    } else if (method == 'b') {
        char contract_address[41] = {0};

        uint64_t ret_code = get_address(contract_address);
        if (ret_code != 0) {
            return ret_code;
        }

        ret = b(contract_address);
    } else {
        pvm_ret_str("method not found");
        return ERROR_METHOD_NOT_FOUND;
    }

    return ret;
}
