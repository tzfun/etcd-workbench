import { StreamLanguage } from '@codemirror/language'
import { shell } from '@codemirror/legacy-modes/mode/shell'

const shellLanguage = () => StreamLanguage.define(shell)
export default shellLanguage