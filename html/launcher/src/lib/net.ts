export async function get<T>(url: string, query: any | undefined = undefined): Promise<T | null> {
    if (query) {
        const qs = new URLSearchParams(query).toString();
        return call<T>(`${url}?${qs}`, { method: "GET" });
    }
    return call<T>(url, { method: "GET" });
}

export async function post<R, T>(url: string, body: R | undefined = undefined): Promise<T | null> {
    return call<T>(url, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: body ? JSON.stringify(body) : undefined,
    });
}

async function call<T>(url: string, opt: RequestInit): Promise<T | null> {
    let host = window.SERVER_URL;
    const res = await fetch(`${host}/api/v1${url}`, opt);
    if (!res.ok) return null;
    try {
        const resp: Resp<T> = await res.json();
        if (resp.code !== 0) return null;
        return resp.data;
    } catch (e) {
        return null;
    }
}

class Resp<T> {
    code: number = 0;
    message: string | null = null;
    data: T | null = null;
}
