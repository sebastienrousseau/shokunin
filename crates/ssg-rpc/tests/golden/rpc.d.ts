// AUTO-GENERATED snapshot — do not edit by hand.

export interface SnapInput {
  author?: string | null;
  post_id: string;
}

export interface SnapOutput {
  likes: number;
}

export interface Rpc {
  snap_like(input: SnapInput): Promise<SnapOutput>;
}
