interface ErrorConstructor {
  captureStackTrace?(object: object, constructor?: () => void): void
}
