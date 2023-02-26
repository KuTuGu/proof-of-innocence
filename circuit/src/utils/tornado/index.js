import { buildBabyjub, buildPedersenHash } from "circomlibjs";

export class TornadoUtil {
  async init() {
    this._babyjub = await buildBabyjub();
    this._pedersen = await buildPedersenHash();
  }

  pedersen_hash(data) {
    // @ts-ignore
    return this._babyjub.F.toObject(this._babyjub.unpackPoint(this._pedersen.hash(data))[0]);
  }
}
