---
type: Policy
title: Política de integração contínua do IES
description: Gates reproduzíveis com prek e regras OKF e tamanho
---

# CI orientado por prek

O arquivo [prek.toml](../prek.toml) é a única fonte de verdade dos gates.

## Comandos

- Pré-commit: `uvx --from prek==0.5.7 prek run --hook-stage pre-commit --all-files`.
- Pré-push e CI remoto: `uvx --from prek==0.5.7 prek run --hook-stage pre-push --all-files`.
- Instalação: `uvx --from prek==0.5.7 prek install --hook-type pre-commit --hook-type pre-push`.

O pré-commit executa tamanho, rustfmt e OKF. O pré-push acrescenta Clippy,
build e testes, todos com Cargo.lock travado.

## OKF

Todo Markdown autoral exige frontmatter com `type` e Spec correspondente
em `okf_specs/{slug}.md`; o gate exige `--normative-spec`.
A versão do parser é fixada em `0.48.2`, ferramenta externa de CI;
o executável IES não depende de Python.

## Arquivos

Máximo de 20.000 bytes UTF-8 por arquivo texto, inclusive `Cargo.lock`.
Arquivos binários são ignorados. Nada de exceções automáticas para gerados.
Arquivos que precisem superar o limite exigem reconsideração explícita da
política — nunca truncamento ou minificação para contornar o check.

## Reexecução

Os hooks devem ser idênticos na estação local e no executor Cloud Build.
O CI remoto não está configurado por este documento: requer conexão do
novo repositório GitHub e execução do mesmo comando do pré-push.
