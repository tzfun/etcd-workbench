import {open} from '@tauri-apps/api/shell'
import {_alertError} from "~/common/events.ts";

const TEXT_DECODER = new TextDecoder();
const TEXT_ENCODER = new TextEncoder();

export type TimeUnit =
    'y'
    | 'm'
    | 'd'
    | 'H'
    | 'M'
    | 'S'
    | 's'
    | 'year'
    | 'month'
    | 'day'
    | 'hour'
    | 'minute'
    | 'second'
    | 'millisecond'
export const fileTypeIcon: Record<string, string> = {
    file: 'mdi-file-document-outline',
    text: 'mdi-file-document-outline',
    js: 'mdi-nodejs',
    ts: 'mdi-language-typescript',
    json: 'mdi-code-json',
    md: 'mdi-language-markdown',
    sql: 'mdi-database-search',
    xml: 'mdi-file-xml-box',
    yaml: 'mdi-code-block-braces',
    properties: 'mdi-cog',
    blob: 'mdi-alpha-b'
}

export function _goBrowserPage(address: string) {
    open(address)
        .then(() => {
        })
        .catch(e => {
            console.error(e)
            _alertError("Open browser failed: {e}")
        })
}

export function _byteTextFormat(bytes: number): string {
    if (bytes < 1024) {
        return bytes + "B"
    } else if (bytes < 1024 * 1024) {
        return (bytes / 1024).toFixed(2) + "KB"
    } else if (bytes < 1024 * 1024 * 1024) {
        return (bytes / (1024 * 1024)).toFixed(2) + "MB"
    } else if (bytes < 1024 * 1024 * 1024 * 1024) {
        return (bytes / (1024 * 1024 * 1024)).toFixed(2) + "GB"
    } else {
        return (bytes / (1024 * 1024 * 1024 * 1024)).toFixed(2) + "TB"
    }
}

export function _timeFormat(timestamp: number, limit: TimeUnit = 'S') {
    let pattern
    if (limit === 'year' || limit === 'y') {
        pattern = "YYYY"
    } else if (limit === 'month' || limit === 'm') {
        pattern = "YYYY-mm"
    } else if (limit === 'day' || limit === 'd') {
        pattern = "YYYY-mm-dd"
    } else if (limit === 'hour' || limit === 'H') {
        pattern = "YYYY-mm-dd HH"
    } else if (limit === 'minute' || limit === 'M') {
        pattern = "YYYY-mm-dd HH:MM"
    } else if (limit === 'second' || limit === 'S') {
        pattern = "YYYY-mm-dd HH:MM:SS"
    } else {
        pattern = "YYYY-mm-dd HH:MM:SS.sss"
    }

    return _dateFormat(pattern, new Date(timestamp))
}

export function _dateFormat(fmt: string, date: Date) {
    let ret;
    const opt = {
        "Y+": date.getFullYear().toString(),        // 年
        "m+": (date.getMonth() + 1).toString(),     // 月
        "d+": date.getDate().toString(),            // 日
        "H+": date.getHours().toString(),           // 时
        "M+": date.getMinutes().toString(),         // 分
        "S+": date.getSeconds().toString(),         // 秒
        "s+": date.getMilliseconds().toString(),    //  毫秒
    };
    for (let k in opt) {
        ret = new RegExp("(" + k + ")").exec(fmt);
        if (ret) {
            //  @ts-ignore
            fmt = fmt.replace(ret[1], (ret[1].length === 1) ? (opt[k]) : (opt[k].padStart(ret[1].length, "0")))
        }
    }
    return fmt;
}

export function _isEmpty(str: string | null | undefined) {
    return !str || str.trim().length == 0;
}

export function _nonEmpty(str: string | null | undefined) {
    return !_isEmpty(str);
}

export function _strArrToNumArr(strArr: string[]): number[] {
    let numArr = []
    for (let s of strArr) {
        let a = parseInt(s)
        if (!isNaN(a)) {
            numArr.push(a)
        }
    }
    return numArr
}

export function _decodeBytesToString(bytes: number[]): string {
    return TEXT_DECODER.decode(Uint8Array.from(bytes))
}

export function _encodeStringToBytes(str?: string): number[] {
    return Array.from(TEXT_ENCODER.encode(str))
}

export function _upperCaseFirst(str: string): string {
    return str.substring(0, 1).toUpperCase() + str.substring(1)
}

export function _shuffleArray(arr: any[]) {
    let len = arr.length;
    for (let i = len - 1; i >= 0; i--) {
        let randomIndex = Math.floor(Math.random() * (i + 1));
        let temp = arr[randomIndex];
        arr[randomIndex] = arr[i];
        arr[i] = temp;
    }
}

/**
 *
 * <pre>
 *     methods: {
 *          _someMethod: _debounce(function () {
 *              //  ...
 *          }, 100)
 *     }
 * </pre>
 *
 * @param fn
 * @param delay
 * @return {(function(): void)|*}
 * @private
 */
export function _debounce(fn: Function, delay: number = 200) {
    let timer = null;
    return function () {
        let _this = this
        let args = arguments
        if (timer) {
            clearTimeout(timer)
        }
        timer = setTimeout(function () {
            fn.apply(_this, args);
        }, delay);
    };
}