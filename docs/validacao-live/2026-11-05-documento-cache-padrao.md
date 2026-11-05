---
type: ValidationReport
title: Validação do cache local padrão dos documentos
operation: documento.obter
date: 2026-11-05
environment: nao-executado
status: NOT_RUN
requests: 0
---

# Ensaio do cache local padrão (#236)

**Estado: NOT_RUN. Requisições: 0.**

Este registro corresponde somente à preparação documental da
[issue #236](https://github.com/franklinbaldo/ies/issues/236).
Não foi feita leitura em instalação de treinamento nem login; nenhum
dado de sessão, senha, cookie ou URL assinada foi coletado.

## Plano de validação

Conferir com XDG_CACHE_HOME temporário uma primeira leitura interna,
a segunda leitura idempotente, versão desconhecida, anexo externo,
`--sem-cache`, `IES_CACHE_DOCUMENTOS=0`, `crc_rodape` presente/ausente
e a cópia em `--saida`. Registrar requisições efetivas e
PASS, FAIL ou NOT_RUN por cenário.

Para futuro ensaio autorizado: usar apenas instalações e contas
públicas de treinamento; nunca repetir requisição que falhou; manter
pelo menos 30 segundos entre comandos e 60 segundos nas PEN; autenticar
uma única vez somente quando necessário. Ante
`bloqueio_de_seguranca_da_instalacao`, encerrar como NOT_RUN.

## Bloqueio

O sandbox local não dispõe de acesso DNS a github.com e não contém
checkout do projeto ou Rust instalado. A preparação usou o conector
GitHub, sem compilação ou execução. Não há resultado RED/GREEN, gate
prek ou CI a declarar para a implementação.
