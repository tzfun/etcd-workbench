import { StreamLanguage } from '@codemirror/language'
import { mySQL } from '@codemirror/legacy-modes/mode/sql'

const sqlLanguage = () => StreamLanguage.define(mySQL)
export default sqlLanguage
