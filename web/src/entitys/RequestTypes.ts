import {SessionConfig} from "~/entitys/TransformTypes";

export interface NewSessionReq extends SessionConfig {
    target: string | null;
}