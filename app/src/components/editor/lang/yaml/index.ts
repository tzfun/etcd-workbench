import { StreamLanguage } from '@codemirror/language'
import { yaml } from '@codemirror/legacy-modes/mode/yaml'

const yamlLanguage = () => StreamLanguage.define(yaml)
export default yamlLanguage