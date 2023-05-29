import {ElLoading} from "element-plus";

export function _loading(msg?: string | undefined) {
    return ElLoading.service({
        lock: true,
        text: msg ? msg : 'Loading',
        background: 'rgba(0, 0, 0, 0.7)',
    })
}