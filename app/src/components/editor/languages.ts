import {EditorMappedLanguage} from "~/common/utils.ts";
import jsonLanguage from "~/components/editor/lang/json";
import xmlLanguage from "~/components/editor/lang/xml";
import yamlLanguage from "~/components/editor/lang/yaml";
import sqlLanguage from "~/components/editor/lang/sql";
import propertiesLanguage from "~/components/editor/lang/properties";
import shellLanguage from "~/components/editor/lang/shell";
import nginxLanguage from "~/components/editor/lang/nginx";
import {EditorHighlightLanguage} from "~/common/types.ts";
import {Extension} from "@codemirror/state";

const importers = import.meta.glob<string>('./lang-code/*/Store.ts')
const languages: { [key in string]: () => any } = {}
Object.keys(importers).forEach((fileName) => {
    const language = fileName.replace('./lang-code/', '').replace('/index.ts', '')
    languages[language] = importers[fileName]
})
export default languages

export function getLanguage(language: EditorHighlightLanguage): Extension | undefined {
    const lang = EditorMappedLanguage[language] || language
    switch (lang) {
        case 'json':
            return jsonLanguage()
        case 'xml':
            return xmlLanguage()
        case 'yaml':
            return yamlLanguage()
        case 'sql':
            return sqlLanguage()
        case 'properties':
            return propertiesLanguage()
        case 'shell':
        case 'dockerfile':
            return shellLanguage()
        case 'nginx':
            return nginxLanguage()
    }
}