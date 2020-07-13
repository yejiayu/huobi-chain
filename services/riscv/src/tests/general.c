//
// Created by lycrus on 7/9/20.
//
#include <pvm.h>
#include <pvm_extend.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>

#define ERROR_METHOD_NOT_FOUND 69

int main(){
    char args[32] = {0};
    uint64_t args_len = pvm_load_args(args);

    if (args_len == 0) {
        pvm_ret_str("method not found");
        return ERROR_METHOD_NOT_FOUND;
    }

    if (pvm_is_init()){
        pvm_ret(args,args_len);
        return 0;
    }

    if(strcmp(args,"pvm_load_args") ==0 ){
        pvm_ret_str("pvm_load_args");
    }else if (strcmp(args,"pvm_ret") ==0 ){
        pvm_ret_str("pvm_ret");
    }else if (strcmp(args,"pvm_cycle_limit") ==0 ){
        pvm_ret_u64_str(pvm_cycle_limit());
    }else if (strcmp(args,"pvm_cycle_used") ==0 ){
        pvm_ret_u64_str(pvm_cycle_used());

    }else if (strcmp(args,"pvm_cycle_price") ==0 ){
        pvm_ret_u64_str(pvm_cycle_price());

    }else if (strcmp(args,"pvm_origin") ==0 ){
        uint8_t *addr = malloc(pvm_origin(NULL));
        uint8_t len = pvm_origin(addr);
        pvm_ret_str((char*)addr);

    }else if (strcmp(args,"pvm_caller") ==0 ){
        uint8_t *addr = malloc(pvm_caller(NULL));
        uint8_t len = pvm_caller(addr);
        pvm_ret_str((char*)addr);

    }else if (strcmp(args,"pvm_address") ==0 ){
        uint8_t *addr = malloc(pvm_address(NULL));
        pvm_address(addr);
        pvm_ret_str((char*)addr);

    }else if (strcmp(args,"pvm_block_height") ==0 ){
        uint64_t block_height = pvm_block_height();
        pvm_ret_u64_str(block_height);
    }else if (strcmp(args,"pvm_extra") ==0 ){
        uint8_t *extra = malloc(pvm_extra(NULL));
        pvm_extra(extra);
        pvm_ret_str((char*)extra);

    }else if (strcmp(args,"pvm_timestamp") ==0 ){
        uint64_t timestamp = pvm_timestamp();

        pvm_ret_u64_str(timestamp);
    }else if (strcmp(args,"pvm_emit_event") ==0 ){
        char name[] = "event_name";
        char data[] = "event_data";
        pvm_emit_event(name,strlen(name),data,strlen(data));
        pvm_ret_str("");
    }else if (strcmp(args,"pvm_tx_hash") ==0 ){
        uint8_t *tx_hash = malloc(pvm_tx_hash(NULL));
        pvm_tx_hash(tx_hash);
        pvm_ret_str((char*)tx_hash);

    }else if (strcmp(args,"pvm_tx_nonce") ==0 ){
        uint8_t *nonce = malloc(pvm_tx_nonce(NULL));
        pvm_tx_nonce(nonce);
        pvm_ret_str((char*)nonce);

    }else{
        pvm_ret_str("not match test case");
        return 69;
    }

    return 0;
}
