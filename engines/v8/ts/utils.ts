import { V8ValueType } from "./types"

export class V8Utils {
  static valueTypeToString(type: V8ValueType): string {
    switch (type) {
      case V8ValueType.Undefined:
        return "undefined"
      case V8ValueType.Null:
        return "null"
      case V8ValueType.Boolean:
        return "boolean"
      case V8ValueType.Number:
        return "number"
      case V8ValueType.String:
        return "string"
      case V8ValueType.Object:
        return "object"
      case V8ValueType.Array:
        return "array"
      case V8ValueType.Function:
        return "function"
      case V8ValueType.Promise:
        return "promise"
      case V8ValueType.Error:
        return "error"
      case V8ValueType.Symbol:
        return "symbol"
      case V8ValueType.BigInt:
        return "bigint"
      case V8ValueType.Proxy:
        return "proxy"
      case V8ValueType.TypedArray:
        return "typedarray"
    }
  }

  static isPrimitiveType(type: V8ValueType): boolean {
    switch (type) {
      case V8ValueType.Undefined:
      case V8ValueType.Null:
      case V8ValueType.Boolean:
      case V8ValueType.Number:
      case V8ValueType.String:
      case V8ValueType.BigInt:
      case V8ValueType.Symbol:
        return true
      default:
        return false
    }
  }
}
