#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Include the generated header
#include "../include/purrmint.h"

int main() {
    printf("Testing PurrMint FFI interface...\n");
    
    // Test 1: Basic FFI test
    printf("\n1. Testing mint_test_ffi()...\n");
    char *test_result = mint_test_ffi();
    if (test_result) {
        printf("Result: %s\n", test_result);
        mint_free_string(test_result);
    } else {
        printf("Error: mint_test_ffi returned NULL\n");
        return 1;
    }
    
    // Test 2: Create Nostr account
    printf("\n2. Testing nostr_create_account()...\n");
    NostrAccount *account = nostr_create_account();
    if (account) {
        printf("Account created successfully\n");
        printf("Public key: %s\n", account->pubkey);
        printf("Is imported: %s\n", account->is_imported ? "true" : "false");
        nostr_free_account(account);
    } else {
        printf("Error: nostr_create_account returned NULL\n");
        return 1;
    }
    
    // Test 3: Get mint info
    printf("\n3. Testing mint_get_info()...\n");
    char *info = mint_get_info();
    if (info) {
        printf("Mint info: %s\n", info);
        mint_free_string(info);
    } else {
        printf("Error: mint_get_info returned NULL\n");
        return 1;
    }
    
    // Test 4: Get mint status
    printf("\n4. Testing mint_get_status()...\n");
    char *status = mint_get_status();
    if (status) {
        printf("Mint status: %s\n", status);
        mint_free_string(status);
    } else {
        printf("Error: mint_get_status returned NULL\n");
        return 1;
    }
    
    printf("\nAll tests passed! FFI interface is working correctly.\n");
    return 0;
} 