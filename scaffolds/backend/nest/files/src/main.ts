import { NestFactory } from '@nestjs/core'
import { ValidationPipe } from '@nestjs/common'
import { SwaggerModule, DocumentBuilder } from '@nestjs/swagger'
import { AppModule } from './app.module'

async function bootstrap() {
  const app = await NestFactory.create(AppModule)

  app.useGlobalPipes(new ValidationPipe({ whitelist: true }))
  app.enableCors()

  const config = new DocumentBuilder()
    .setTitle('{{ name }}')
    .setDescription('{{ description }}')
    .setVersion('{{ version }}')
    .build()

  const document = SwaggerModule.createDocument(app, config)
  SwaggerModule.setup('docs', app, document)

  await app.listen(process.env.PORT || 3000)
  console.log(`{{ name }} running on port ${process.env.PORT || 3000}`)
}

bootstrap()
