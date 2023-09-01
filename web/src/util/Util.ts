import {ElLoading} from "element-plus";

export function _loading(msg?: string | undefined) {
    return ElLoading.service({
        lock: true,
        text: msg ? msg : 'Loading',
        background: 'rgba(0, 0, 0, 0.7)',
    })
}

export function _isEmpty(value: string | undefined | object | null): boolean {
    return (
        value === undefined ||
        value === null ||
        (typeof value === "string" && value.trim().length === 0) ||
        (typeof value === "object" && Object.keys(value).length === 0)
    );
}

export function _nonEmpty(value: string | undefined | object | null): boolean {
    return !_isEmpty(value)
}

export function _sizeof(value: string): number {
    return new TextEncoder().encode(value).length
}

export function _byteFormat(bytes: number): string {
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

/**
 * string转byte数组，UTF-8格式
 *
 * @param str
 * @returns {any[]}
 * @private
 */
export function _strToBytes(str: string) {
    let bytes = [];
    for (let i = 0; i < str.length; i++) {
        let c = str.charCodeAt(i);
        let s = parseInt(c.toString()).toString(2);
        if (c >= parseInt("000080", 16) && c <= parseInt("0007FF", 16)) {
            let af = "";
            for (let j = 0; j < (11 - s.length); j++) {
                af += "0";
            }
            af += s;
            let n1 = parseInt("110" + af.substring(0, 5), 2);
            let n2 = parseInt("110" + af.substring(5), 2);
            if (n1 > 127) n1 -= 256;
            if (n2 > 127) n2 -= 256;
            bytes.push(n1);
            bytes.push(n2);
        } else if (c >= parseInt("000800", 16) && c <= parseInt("00FFFF", 16)) {
            let af = "";
            for (let j = 0; j < (16 - s.length); j++) {
                af += "0";
            }
            af += s;
            let n1 = parseInt("1110" + af.substring(0, 4), 2);
            let n2 = parseInt("10" + af.substring(4, 10), 2);
            let n3 = parseInt("10" + af.substring(10), 2);
            if (n1 > 127) n1 -= 256;
            if (n2 > 127) n2 -= 256;
            if (n3 > 127) n3 -= 256;
            bytes.push(n1);
            bytes.push(n2);
            bytes.push(n3);
        } else if (c >= parseInt("010000", 16) && c <= parseInt("10FFFF", 16)) {
            let af = "";
            for (let j = 0; j < (21 - s.length); j++) {
                af += "0";
            }
            af += s;
            let n1 = parseInt("11110" + af.substring(0, 3), 2);
            let n2 = parseInt("10" + af.substring(3, 9), 2);
            let n3 = parseInt("10" + af.substring(9, 15), 2);
            let n4 = parseInt("10" + af.substring(15), 2);
            if (n1 > 127) n1 -= 256;
            if (n2 > 127) n2 -= 256;
            if (n3 > 127) n3 -= 256;
            if (n4 > 127) n4 -= 256;
            bytes.push(n1);
            bytes.push(n2);
            bytes.push(n3);
            bytes.push(n4);
        } else {
            bytes.push(c & 0xff);
        }
    }
    return bytes;
}

/**
 * bytes转字符串 UTF-8格式
 *
 * @param utf8Bytes
 * @returns {string}
 * @private
 */
export function _bytesToStr(utf8Bytes: number[] | string[]) {
    let unicodeStr = "";

    function readPos(pos: number): number {
        let b = utf8Bytes[pos];
        if (typeof b === 'string') {
            b = parseInt(b)
            utf8Bytes[pos] = b
        }
        return b
    }

    for (let pos = 0; pos < utf8Bytes.length;) {

        let flag = readPos(pos)
        let unicode = 0;
        if ((flag >>> 7) === 0) {
            unicodeStr += String.fromCharCode(readPos(pos));
            pos += 1;

        } else if ((flag & 0xFC) === 0xFC) {
            unicode = (readPos(pos) & 0x3) << 30;
            unicode |= (readPos(pos + 1) & 0x3F) << 24;
            unicode |= (readPos(pos + 2) & 0x3F) << 18;
            unicode |= (readPos(pos + 3) & 0x3F) << 12;
            unicode |= (readPos(pos + 4) & 0x3F) << 6;
            unicode |= (readPos(pos + 5) & 0x3F);
            unicodeStr += String.fromCharCode(unicode);
            pos += 6;

        } else if ((flag & 0xF8) === 0xF8) {
            unicode = (readPos(pos) & 0x7) << 24;
            unicode |= (readPos(pos + 1) & 0x3F) << 18;
            unicode |= (readPos(pos + 2) & 0x3F) << 12;
            unicode |= (readPos(pos + 3) & 0x3F) << 6;
            unicode |= (readPos(pos + 4) & 0x3F);
            unicodeStr += String.fromCharCode(unicode);
            pos += 5;

        } else if ((flag & 0xF0) === 0xF0) {
            unicode = (readPos(pos) & 0xF) << 18;
            unicode |= (readPos(pos + 1) & 0x3F) << 12;
            unicode |= (readPos(pos + 2) & 0x3F) << 6;
            unicode |= (readPos(pos + 3) & 0x3F);
            unicodeStr += String.fromCharCode(unicode);
            pos += 4;

        } else if ((flag & 0xE0) === 0xE0) {
            unicode = (readPos(pos) & 0x1F) << 12;
            unicode |= (readPos(pos + 1) & 0x3F) << 6;
            unicode |= (readPos(pos + 2) & 0x3F);
            unicodeStr += String.fromCharCode(unicode);
            pos += 3;

        } else if ((flag & 0xC0) === 0xC0) { //110
            unicode = (readPos(pos) & 0x3F) << 6;
            unicode |= (readPos(pos + 1) & 0x3F);
            unicodeStr += String.fromCharCode(unicode);
            pos += 2;

        } else {
            unicodeStr += String.fromCharCode(readPos(pos));
            pos += 1;
        }
    }
    return unicodeStr;
}

export function _bytesToHex(bytes: number[]) {
    return bytes.length === 0
        ? ''
        : ('\\x' + Array.from(bytes, (byte) => {
                return ('0' + (byte & 0xFF).toString(16)).slice(-2);
            }).join('\\x')
        )
}

export function _hexToBytes(hex: string) {
    let bytes = []
    for (let c = 0; c < hex.length; c += 2)
        bytes.push(_parseByte(parseInt(hex.substr(c, 2), 16)));
    return bytes;
}

export function _strToHex(str: string) {
    return _bytesToHex(_strToBytes(str))
}

export function _hexToStr(str: string) {
    return _bytesToStr(_hexToBytes(str))
}

function _parseByte(byte: number) {
    let ret = parseInt(byte.toString())
    if (ret > 127) {
        ret -= 256;
    } else if (ret < -128) {
        ret += 256;
    }
    return ret
}
