export function _isEmpty(value: string | undefined | object): boolean {
    return (
        value === undefined ||
        value === null ||
        (typeof value === "string" && value.trim().length === 0) ||
        (typeof value === "object" && Object.keys(value).length === 0)
    );
}

export function _nonEmpty(value: string | undefined | object): boolean {
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
