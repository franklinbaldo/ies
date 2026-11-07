---
type: SEIObservation
title: Origem privada com versão declarada no ensaio de processo criar
description: Compatibilização do ensaio com o formato de origem do IES, RED offline e limites live
operation: processo.criar
date: 2026-11-07
status: not-run
---

# 🦉 #245 — origem de treinamento com versão declarada

## Evidência e defeito

A implementação canônica em `crates/ies-core/src/instalacao.rs`,
`Instalacao::interpretar`, aceita na origem privada a URL HTTPS na
primeira linha e opcionalmente `versao=4.0.9` na segunda, desde que
a versão seja numérica de dois a quatro componentes. Esse formato
também é descrito em `docs/unidade-primeira-leitura.md`.

A guarda do ensaio manual `scripts/ensaio_processo_criar.py`
comparava **todo o conteúdo do arquivo** à lista de bases de treinamento.
Com a segunda linha legítima, recusava a origem antes da prévia, embora
a CLI reconhecesse a mesma instalação. A correção compara **somente a
base HTTPS** à allowlist das sete instalações, valida a versão pelo
formato declarado e recusa linhas extras/metadados alheios. Preserva
a leitura privada 0600 e a vedação a `IES_CONFIG_FILE` concorrente.

## RED → GREEN

- Teste `test_wip_245_origem_sei_com_versao_declarada_eh_permitida`
  registrado antes da implementação em `88154d0`.
- [CI RED 42368252540](https://github.com/franklinbaldo/ies/actions/runs/42368252540),
  job `replay processo criar`: **26 testes Python, um ERROR novo**,
  `ValueError: origem não consta das instalações de treinamento`.
- Correção no ensaio em `683adaa`; também foi incluído teste para
  rejeitar versão malformada, linha adicional e metadado desconhecido.
- Esperado: liberação de origem legítima e recusa das inválidas, sem
  alterar requisições da CLI, sem retry e sem permitir produção. O
  resultado GREEN é conferido no CI oficial do SHA final da PR.

## Evidência live

A prévia autenticada numa segunda instalação de treinamento, o único
`--confirmar` e a releitura independente permanecem **NOT_RUN**.
Nesta fatia: **0 requisições reais ao SEI, 0 POST, 0 escritas e 0
resíduos novos**. O sandbox não dispõe de DNS nem de sessão autenticada
pronta para concluir o ciclo com segurança.

A alteração não prova causalidade de um POST incerto, nem autoriza
repetir escrita. A issue #245 permanece WIP até realizar o ensaio
inteiro com NUP, contagem real e resíduo documentados.
