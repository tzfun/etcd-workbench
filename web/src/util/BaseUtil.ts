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