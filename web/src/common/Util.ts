import {ElLoading} from "element-plus";
import JSEncrypt from 'jsencrypt'
import md5 from 'js-md5'

let loading: any = null
let loadingCounter: number = 0

export function _startLoading(msg?: string | undefined) {
    if (loadingCounter == 0) {
        loading = ElLoading.service({
            lock: true,
            text: msg ? msg : 'Loading',
            background: 'rgba(0, 0, 0, 0.7)',
        })
    }
    loadingCounter++
}

export function _endLoading() {
    if (loadingCounter > 0) {
        loadingCounter--
    }
    if (loadingCounter == 0 && loading) {
        loading.close()
    }
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

function _parseByte(byte: number): number {
    if (byte > 127) {
        byte -= 256;
    } else if (byte < -128) {
        byte += 256;
    }
    return byte
}

/**
 * string转byte数组，UTF-8格式
 *
 * @param str
 * @returns {any[]}
 * @private
 */
export function _strToBytes(str: string): number[] {
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
            bytes.push(_parseByte(n1));
            bytes.push(_parseByte(n2));
        } else if (c >= parseInt("000800", 16) && c <= parseInt("00FFFF", 16)) {
            let af = "";
            for (let j = 0; j < (16 - s.length); j++) {
                af += "0";
            }
            af += s;
            let n1 = parseInt("1110" + af.substring(0, 4), 2);
            let n2 = parseInt("10" + af.substring(4, 10), 2);
            let n3 = parseInt("10" + af.substring(10), 2);
            bytes.push(_parseByte(n1));
            bytes.push(_parseByte(n2));
            bytes.push(_parseByte(n3));
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
            bytes.push(_parseByte(n1));
            bytes.push(_parseByte(n2));
            bytes.push(_parseByte(n3));
            bytes.push(_parseByte(n4));
        } else {
            bytes.push(_parseByte(c & 0xff));
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
export function _bytesToStr(utf8Bytes: number[] | string[]): string {
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

/**
 * RSA加密
 *
 * @param str       明文
 * @param pubKey    公钥
 * @returns {string | false}
 */
export function _rsaEncrypt(str: string, pubKey: string): string | false {
    let encryptStr = new JSEncrypt({
        log: true
    });
    encryptStr.setPublicKey(pubKey);
    return encryptStr.encrypt(str);
}

/**
 * rsa加密有长度限制，如果加密文本长度超出长度限制则进行分段加密
 *
 * @param str       明文
 * @param pubKey    公钥
 * @param splitter  密文分割字符
 * @returns {string|false}
 */
export function _rsaEncryptPartly(str: string, pubKey: string, splitter: string): string | false {
    const maxLen = 117  //  最大字节数
    let bytes = _strToBytes(str)
    const strLen = bytes.length

    if (strLen <= maxLen) {
        return _rsaEncrypt(str, pubKey)
    } else {
        let idx = 0
        let rsaArr = []
        while (idx < strLen) {
            let part = bytes.slice(idx, Math.min(strLen, idx + maxLen))
            const partResult = _rsaEncrypt(_bytesToStr(part), pubKey)
            if (!partResult) {
                return false
            }
            rsaArr.push(partResult)
            idx += Math.min(maxLen, strLen - idx)
        }
        return rsaArr.join(splitter)
    }
}

export function _md5(obj: string): string {
    let hash = md5.create()
    hash.update(obj)
    return hash.hex()
}

export function _parseCodeLanguage(filename: string, content: string): string {
    let lang;
    if (filename) {
        filename = filename.toLowerCase()
        let splitter = filename.split(".")
        if (splitter.length > 0) {
            let fileType = splitter[splitter.length - 1]
            switch (fileType) {
                case "xml":
                case "html":
                case "htm":
                    lang = 'xml'
                    break
                case "json":
                    lang = 'json'
                    break
                case "conf":
                case "properties":
                    lang = 'properties'
                    break
                case "yaml":
                case "yml":
                    lang = "yaml"
                    break
                case "sql":
                    lang = "sql"
                    break
            }
        }
    }
    if (!lang && content) {
        if (content.startsWith('<')) {
            lang = 'xml'
        } else if (content.startsWith('{') || content.startsWith('[')) {
            lang = 'json'
        } else if (content.startsWith('---')) {
            lang = 'yaml'
        } else if (content.startsWith("--")) {
            lang = "sql"
        }
    }
    if (!lang) {
        lang = 'text'
    }

    console.log("parse lang ", filename, lang)
    return lang
}

export function _saveFile(blob: Blob, name: string) {
    _startLoading("Downloading...")
    let urlObject = window.URL || window.webkitURL || window;
    let url = urlObject.createObjectURL(blob);
    _download(url, name)
    _endLoading()
}

export function _download(url: string, filename: string) {
    let urlObject = window.URL || window.webkitURL || window
    let el = document.createElement("a")
    el.href = url
    el.download = filename
    el.click()
    urlObject.revokeObjectURL(url)
}
