export const config = {
  port: parseInt(process.env.PORT || '3000', 10),
  host: process.env.HOST || '0.0.0.0',
  nodeEnv: process.env.NODE_ENV || 'development',
  logLevel: process.env.LOG_LEVEL || 'info',
  corsOrigins: process.env.CORS_ORIGINS || '*',
  swagger: {
    title: process.env.SWAGGER_TITLE || '{{ name }}',
    version: process.env.SWAGGER_VERSION || '{{ version }}',
    description: process.env.SWAGGER_DESCRIPTION || '{{ description }}',
  },
}
