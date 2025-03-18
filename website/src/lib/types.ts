export type Streamed<T> = T | Promise<T>;

export type ListResponse<T> = {
    count: number;
    next: string | null;
    previous: string | null;
    results: T[];
};
