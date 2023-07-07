export interface NewSessionReq extends Record<string, any> {
    namespace: string | null;
    target: string | null;
    user?: string | null;
    password?: string | null;
    authority?: string | null;
    caType: string | 'none';
    caCert?: string | null;
    clientCertMode?: string | 'none';
    clientCert?: string | null;
    clientCertPassword?: string | null;
    clientCertKey?: string | null;
}