const BASE_URL = "http://127.0.0.1:12345/api";

export async function get<T>(url: string): Promise<T | null> {
    return call<T>(url, { method: "GET" });
}

export async function post<R, T>(url: string, body: R | undefined = undefined): Promise<T | null> {
    return call<T>(url, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: body ? JSON.stringify(body) : undefined
    });
}

async function call<T>(url: string, opt: RequestInit): Promise<T | null> {
    const res = await fetch(`${BASE_URL}/v1${url}`, opt);
    if (!res.ok) {
        return null;
    }
    const resp: Resp<T> = await res.json();
    if (resp.code !== 0) {
        return null;
    }
    return resp.data;
}

class Resp<T> {
    code: number = 0;
    message: string | null = null;
    data: T | null = null;
}