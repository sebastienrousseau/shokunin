// AUTO-GENERATED snapshot — do not edit by hand.

export interface SnapLikeInput {
  author?: string | null;
  post_id: string;
}

export interface SnapLikeOutput {
  likes: number;
}

export interface Rpc {
  snap_like(input: SnapLikeInput): Promise<SnapLikeOutput>;
}
