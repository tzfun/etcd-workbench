import {open} from '@tauri-apps/api/shell'
import {_alertError} from "~/common/events.ts";
import {EditorHighlightLanguage} from "~/common/types.ts";
import {_useGlobalStore} from "~/common/store.ts";

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
    blob: 'mdi-alpha-b',
    shell: 'mdi-powershell',
    dockerfile: 'mdi-docker',
    nginx: 'mdi-alpha-n-box'
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
    let timer: NodeJS.Timeout | null = null;
    return function () {
        //  @ts-ignore
        let _this: any = this
        let args = arguments
        if (timer) {
            clearTimeout(timer)
        }
        timer = setTimeout(function () {
            fn.apply(_this, args);
        }, delay);
    };
}

export function _pointInRect(point: { x: number, y: number }, rect: DOMRect) {
    const {x, y} = point;
    const dx = rect.x, dy = rect.y, width = rect.width, height = rect.height;
    return x >= dx && x <= dx + width && y >= dy && y <= dy + height;
}

export function _tryParseEditorLanguage(key: string, content: number[] | string, namespace?: string): EditorHighlightLanguage {
    //  先从记录中读取用户选择的格式
    let fullKey = namespace ? (namespace + key) : key

    let rememberedFormat = _useGlobalStore().value.fileFormatLogMap[fullKey]
    if (rememberedFormat) {
        console.debug("Read remembered format", fullKey, "==>", rememberedFormat)
        return rememberedFormat
    } else {
        let language = _tryParseEditorLanguageByName(key)
        if (!language) {
            let contentStr = typeof content == 'string' ? content : _decodeBytesToString(content)
            language = _tryParseEditorLanguageByContent(contentStr)
        }
        return language
    }
}

export function _tryParseDiffLanguage(editorLanguage: EditorHighlightLanguage): string {
    switch (editorLanguage) {
        case 'text':
            return 'plaintext'
        case 'sql':
            return 'SQL'
        case 'markdown':
            return 'Markdown'
        case 'xml':
            return 'HTML'
        case 'json':
            return 'Json'
        case 'yaml':
            return 'YAML'
        default:
            return 'plaintext'
    }
}

function _tryParseEditorLanguageByContent(content: string): EditorHighlightLanguage {
    let lang: EditorHighlightLanguage = 'text'
    content = content.trimStart()
    if (content.startsWith('<')) {
        lang = 'xml'
    } else if (content.startsWith('{') || content.startsWith('[')) {
        lang = 'json'
    } else if (content.startsWith('---')) {
        lang = 'yaml'
    } else if (content.startsWith("--")) {
        lang = "sql"
    } else if (content.startsWith("#!")) {
        lang = "shell"
    }
    return lang
}

function _tryParseEditorLanguageByName(fileName: string, defaultType?: EditorHighlightLanguage): EditorHighlightLanguage | undefined {
    let dotIdx = fileName.lastIndexOf(".")
    if (dotIdx >= 0) {
        let type = fileName.substring(dotIdx + 1).toLowerCase()
        switch (type) {
            case 'json':
                return 'json'
            case 'sql':
                return 'sql'
            case 'xml':
            case 'html':
            case 'htm':
                return 'xml'
            case 'yml':
            case 'yaml':
                return 'yaml'
            case 'ts':
            case 'typescript':
                return 'ts'
            case 'js':
            case 'javascript':
                return 'js'
            case 'md':
            case 'markdown':
                return 'markdown'
            case 'ini':
            case 'properties':
                return 'properties'
            case 'conf':
            case 'nginx':
            case 'nginxconf':
                return 'nginx'
            case 'dockerfile':
            case 'docker':
                return 'dockerfile'
            case 'sh':
                return 'shell'
            default:
                return defaultType
        }
    }

    return defaultType
}
