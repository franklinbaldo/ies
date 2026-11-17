---
type: SEIObservation
title: Validação do cache local padrão dos documentos na UnB
description: Ensaios ao vivo de cache interno, externo PDF, idempotência, saída e desligamento na UnB 4.0.9, com 28 requisições contabilizadas
---

# Cache privado de documentos — UnB 4.0.9 (#236)

**PASS live** dos cenários executados: documento interno, PDF externo,
idempotência, derivação de `--saida`, CRC observado e desligamento
por flag e ambiente. Dois ensaios de 05/11/2026:
[primeiro](https://github.com/franklinbaldo/ies/actions/runs/42065833103)
e [ampliado](https://github.com/franklinbaldo/ies/actions/runs/42066983084).

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
  de ao menos 30 s entre comandos. Foram 8 comandos, com
  28 requisições informadas pelo IES nas duas execuções, sem
  contagem independente por proxy.

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

## Ensaio ampliado: saída, flag e PDF externo

Workflow [#42066983084](https://github.com/franklinbaldo/ies/actions/runs/42066983084),
com nova sessão efêmera e o mesmo código já mesclado na main.
Credenciais públicas lidas do guia, materializadas só em arquivo 0600.

| Caso | Requisições | Status | Evidência |
| --- | ---: | --- | --- |
| Login | 6 | PASS | Uma autenticação na UnB |
| `0011870 --saida DIR` | 3 | PASS | `arquivo.sha256` igual ao cache, bytes da foto iguais ao SHA-256 do cache; `crc_rodape` presente e versão 3 |
| `0011870 --sem-cache` | 2 | PASS | `cache: null` e mesmo conjunto de arquivos antes/depois |
| Externo `0011537` PDF | 4 | PASS | Cache `.pdf`, 14.880 bytes, SHA-256 integral e permissão privada conferidos |

Total da rodada: **15 requisições**. Somadas às 13 da primeira,
**28 requisições informadas pelo IES**, 8 comandos e nenhum retry.
Não houve escrita remota. O workflow foi removido depois de concluído
e não foi integrado à main.

## Não exercitado ao vivo

- **NOT_RUN:** documento externo TXT/HTML, versão desconhecida
  (nome `x-...` no HTML interno), `crc_rodape` ausente.
- A presença de CRC foi observada, não testada sua autenticidade:
  o CRC é apenas informativo.
- Esses casos restantes têm cobertura sintética no código; não
  afirmar validação live onde não houve comando correspondente.
- As contagens foram informadas pelo IES, não verificadas com
  proxy independente nesta rodada.
- A compilação ocorreu no Actions; o sandbox desta sessão não
  tinha Rust nem DNS para checkout local.
- O RED inicial dos testes da PR #240 não teve contagem conclusiva;
  isso não foi corrigido retroativamente.

## Limpeza

Ambos os workflows de teste eram temporários, restritos às branches
`codex/236-cache-documentos` e `codex/236-live-ampliado`.
Foram removidos depois dos respectivos PASS, sem integrar à `main`;
nenhuma credencial de sessão foi comitada. A verificação do gate oficial permanece registrada na
[PR #240](https://github.com/franklinbaldo/ies/pull/240)
e na [PR #252](https://github.com/franklinbaldo/ies/pull/252).
