onmessage = async function(e) {
    let thread_info = e.data;
    
    function chars_to_string(chars_ptr, len) {
        let out = "";
        let array = new Uint32Array(thread_info.memory.buffer, chars_ptr, len);
        for (let i = 0; i < len; i ++) {
            out += String.fromCharCode(array[i]);
        }
        return out
    }
    
    let env = {
        memory: thread_info.memory,
        
        js_console_error: (chars_ptr, len) => {
            console.error(chars_to_string(chars_ptr, len))
        },
        
        js_console_log: (chars_ptr, len) => {
            console.log(chars_to_string(chars_ptr, len))
        },
        js_post_signal: (signal_hi, signal_lo) => {
            postMessage({
                message_type: "signal",
                signal_hi,
                signal_lo
            });
        }
    };
    
    WebAssembly.instantiate(thread_info.bytes, {env}).then(wasm => {
        
        wasm.instance.exports.__stack_pointer.value = thread_info.stack_ptr;
        wasm.instance.exports.__wasm_init_tls(thread_info.tls_ptr);
        
        wasm.instance.exports.wasm_thread_entrypoint(thread_info.closure_ptr);
        
        close();
    }, error => {
        console.error("Cannot instantiate wasm" + error);
    })
}
