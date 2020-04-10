#include <pvm.h>
#include <pvm_extend.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

uint64_t test_service_call_read_fail() {
    const char* service = "asset";
    const char* method = "get_balance";
    const char* payload = "{\"asset_id\":\"0xf56924db538e77bb5951eb5ff0d02b88983c49c45eea30e8ae3e7234b311436c\", \"user\":\"0xf8389d774afdad8755ef8e629e5a154fddc6325a\"}";

    uint8_t ret[1000] = {0};
    uint64_t ret_len = pvm_service_call(service, method,
                          payload,  strlen(payload),
                          ret);
    pvm_debug(ret);
    pvm_ret(ret, ret_len);
    return 0;
}

uint64_t test_service_read() {
    const char* service = "asset";
    const char* method = "get_balance";
    const char* payload = "{\"asset_id\":\"0xf56924db538e77bb5951eb5ff0d02b88983c49c45eea30e8ae3e7234b311436c\", \"user\":\"0xf8389d774afdad8755ef8e629e5a154fddc6325a\"}";

    uint8_t ret[1000] = {0};
    uint64_t ret_len = pvm_service_read(service, method,
                          payload,  strlen(payload),
                          ret);
    pvm_debug(ret);
    pvm_ret(ret, ret_len);
    return 0;
}

uint64_t test_transfer_from_contract() {
    const char* service = "asset";
    const char* method = "transfer";
    const char* payload = "{\"asset_id\":\"0xf56924db538e77bb5951eb5ff0d02b88983c49c45eea30e8ae3e7234b311436c\", \"to\":\"0x0000000000000000000000000000000000000001\", \"value\": 100}";

    uint8_t ret[1000] = {0};
    uint64_t ret_len = pvm_service_call(service, method,
                          payload,  strlen(payload),
                          ret);
    pvm_debug(ret);
    pvm_ret(ret, ret_len);
    return 0;
}

uint64_t dummy_method() {
    const char* ret = "{\"a\":1}";
    pvm_debug(ret);
    pvm_ret(ret, strlen(ret));
    return 0;
}

uint64_t test_call_dummy_method() {
    uint8_t addr[50] = {0};
    pvm_address(addr);
    const char *args = "dummy_method";
    uint8_t ret[2048] = {0};
    uint64_t size = pvm_contract_call(addr, (uint8_t *)args, strlen(args), ret);
    pvm_debug(ret);
    pvm_ret(ret, strlen(ret));
    return 0;
}

int main() {
    char args[1024] = {0};
    uint64_t args_len = pvm_load_args(args);
    pvm_debug(args);

    uint64_t ret = 0;
    if (strcmp(args, "test_service_read") == 0) {
        ret = test_service_read();
    } else if (strcmp(args, "test_service_call_read_fail") == 0) {
        ret = test_service_call_read_fail();
    } else if (strcmp(args, "test_transfer_from_contract") == 0) {
        ret = test_transfer_from_contract();
    } else if (strcmp(args, "dummy_method") == 0) {
        ret = dummy_method();
    } else if (strcmp(args, "test_call_dummy_method") == 0) {
        ret = test_call_dummy_method();
    } else {
        pvm_ret_str("method not found");
        return 1;
    }

	return ret;
}

