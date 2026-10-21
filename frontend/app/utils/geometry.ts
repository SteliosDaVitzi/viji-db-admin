export function isSpatialType(dataType: string): boolean {
  return /^(geometry|geography)(\s*\(.*\))?$/i.test(dataType.trim())
}
