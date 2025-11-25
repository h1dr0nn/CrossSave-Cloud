export class FakeR2Bucket {
  private storage = new Map<string, string>();

  async get(key: string): Promise<R2ObjectBody | null> {
    if (!this.storage.has(key)) {
      return null;
    }
    const body = this.storage.get(key) as string;
    const encoder = new TextEncoder();
    const data = encoder.encode(body);
    return {
      arrayBuffer: async () => data.buffer.slice(0),
      json: async <T>() => JSON.parse(body) as T,
      text: async () => body
    } as unknown as R2ObjectBody;
  }

  async head(key: string): Promise<R2Object | null> {
    if (!this.storage.has(key)) {
      return null;
    }
    return { key } as unknown as R2Object;
  }

  async put(key: string, value: string | ArrayBuffer | ArrayBufferView): Promise<void> {
    if (typeof value === "string") {
      this.storage.set(key, value);
      return;
    }
    let buffer: ArrayBuffer;
    if (value instanceof ArrayBuffer) {
      buffer = value;
    } else {
      buffer = value.buffer.slice(value.byteOffset, value.byteOffset + value.byteLength);
    }
    const decoder = new TextDecoder();
    this.storage.set(key, decoder.decode(buffer));
  }
}
