import { StreamLanguage } from '@codemirror/language'
import { xml } from '@codemirror/legacy-modes/mode/xml'

const xmlLanguage = () => StreamLanguage.define(xml)
export default xmlLanguage