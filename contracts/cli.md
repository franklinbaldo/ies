---
type: Contract
title: Contrato da CLI IES
description: Regras publicas provisórias da interface de linha de comando
---

# Contrato CLI do IES — fundação provisória

A interface CLI é a especificação primária; a implementação será guiada pelos
testes e pelas instalações reais.

## Invariantes

- Interface exclusivamente CLI, em português, orientada por substantivos e verbos de domínio.
- `ies <substantivo> NUMERO` lê; `ies <substantivo>` sem argumentos mostra ajuda.
- Não existe `agir`, `Action` ou `PageModel` como contrato público.
- `--json` e `--unidade` são globais; falhas operacionais são estruturadas.
- Códigos de saída: 0 sucesso, 1 falha, 2 uso inválido, 3 efeito incerto.
- Escritas exigem prévia, confirmação e verificação; efeito incerto nunca se repete automaticamente.
- Comportamentos são mapeados empiricamente em instalações reais do SEI.
- Esta superfície de demonstração não é integral: o inventário será aprovado comando a comando.

A [matriz de evidências](evidencias/README.md) relaciona operações
registradas pelo Todos a cassettes, mas **não equivale a aprovação dos
contratos desta CLI**. Contrato e evidência são fontes distintas.

## Primeiro recorte, sem acesso ao SEI

- `ies processo NUP`
- `ies documento NUMERO`
- `ies unidade listar-processos`
- `ies processo enviar NUP --para SIGLA`
- `ies documento assinar NUMERO`
- `ies status`
- `ies setup`
- `ies esquema`
