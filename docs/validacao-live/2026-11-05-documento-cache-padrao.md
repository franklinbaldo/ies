---
type: SEIObservation
title: Validação do cache local padrão dos documentos na UnB
description: Ensaio ao vivo de cache de documento interno, idempotência e desligamento por ambiente na UnB 4.0.9, com 13 requisições contabilizadas
---

# Cache privado de documentos — UnB 4.0.9 (#236)

**PASS parcial live** para documento interno, releitura idempotente e
desligamento por variável de ambiente. Execução de 05/11/2026:
[GitHub Actions #42065833103](https://github.com/franklinbaldo/ies/actions/runs/42065833103).

## Contexto e limites

- Binário compilado em `ubuntu-24.04` com Rust 1.99.1 a partir
  de `codex/236-cache-documentos`, código integrado na
  [PR #252](https://github.com/franklinbaldo/ies/pull/252).
- Instalação pública UnB 4.0.9; conta pública de treinamento da
  [tabela oficial](../interfaces/credenciais-de-treinamento.md).
- Documento interno `0011870`, processo `23106.001748/2026-94`.
- Sessão e credenciais só em arquivos efêmeros `0600`; diretórios
  do ensaio `0700`; não imprimir segredo, cookie, URL assinada
  ou `infra_hash`. `XDG_CACHE_HOME` temporário.
- Somente leitura; nenhuma escrita remota, nenhum retry; intervalos
  de ao menos 30 s entre comandos. Orçamento usado: 4 comandos,
  13 requisições informadas pelo IES, sem contagem independente
  por proxy nesta execução.

## Resultados

| Caso | Requisições | Status | Evidência |
| --- | ---: | --- | --- |
| Login único | 6 | PASS | `login` exit 0 |
| Primeira leitura, cache padrão | 3 | PASS | `cache.caminho` existe, versão 3; SHA-256 integral confere com bytes e arquivo privado |
| Segunda leitura do mesmo DOC | 2 | PASS | Mesmo caminho e SHA-256; `mtime_ns` não mudou |
| `IES_CACHE_DOCUMENTOS=0` | 2 | PASS | Leitura exit 0 e `cache: null` |

Total: **13 requisições contabilizadas** e **zero chamadas repetidas
após falha**. O primeiro ensaio, no workflow
[#42065534050](https://github.com/franklinbaldo/ies/actions/runs/42065534050),
parou **antes da rede**, com `origem_nao_autorizada` e zero
requisições: o leitor da tabela deixou os delimitadores Markdown
de código no endereço da instalação. O parse foi corrigido no
workflow temporário e a nova execução teve PASS. Esse erro era do
harness do ensaio, não da implementação do IES.

## Não exercitado ao vivo

- **NOT_RUN:** documento externo PDF/TXT/HTML, versão desconhecida,
  `--sem-cache` (flag), `crc_rodape` presente/ausente e
  `--saida DIR` como derivação do cache.
- Esses comportamentos têm cobertura sintética no código; não
  afirmar validação live onde não houve comando correspondente.
- A contagem não foi verificada por túnel/proxy independente.
- A compilação ocorreu no Actions; o sandbox desta sessão não
  tinha Rust nem DNS para um checkout local.
- O RED inicial dos testes da PR #240 não teve contagem conclusiva;
  isso não foi corrigido retroativamente.

## Limpeza

O workflow de teste era temporário e restrito à branch
`codex/236-cache-documentos`. Foi removido depois do PASS,
sem integrar à `main`; nenhuma credencial de sessão foi
comitada. A verificação do gate oficial permanece registrada na
[PR #240](https://github.com/franklinbaldo/ies/pull/240)
e na [PR #252](https://github.com/franklinbaldo/ies/pull/252).
