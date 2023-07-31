import {SessionConfig} from "~/entitys/TransformTypes";

export interface NewSessionReq extends SessionConfig, Record<string, any> {
    target: string | null;
}