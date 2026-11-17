---
type: SEIObservation
title: "Prévia de remoção de atribuição em Goiás — segunda instalação de treinamento"
description: "Ensaio somente leitura de processo atribuir --remover em Goiás 5.0.3, com login e prévia PASS, contagem real, sem confirmação nem escrita"
---

# Issue #264 — prévia GO, 06/11/2026

Ensaio na instalação **GO 5.0.3 de TREINAMENTO**, exclusivamente com a
conta pública e a origem fixa de
[credenciais-de-treinamento.md](../interfaces/credenciais-de-treinamento.md).
Evidência executável: [GitHub Actions #42308660667](https://github.com/franklinbaldo/ies/actions/runs/42308660667),
head do ensaio `eff4224e`; job `previa_go` **SUCCESS**.
Foi usado o Rust 1.99.1 e `cargo build --release --locked --bin ies`,
com compilação **PASS**.

## Procedimento seguro

- Origem conferida expressamente no executor: somente
  `https://seitreinamento.go.gov.br/sei` (não produção).
- Origem, credenciais, cookie e Controle foram gravados em arquivos
  privados com `umask 077` e criação `0600`; nenhuma senha, URL assinada,
  cookie ou `infra_hash` foi publicada.
- `ies --json login` **uma vez**; após o comando, intervalo de
  **95 segundos** para respeitar o ritmo de Goiás.
- `ies --json processo atribuir 202400013000003 --remover` **uma vez,
  sem `--confirmar`**. Processo previamente identificado como aberto
  na unidade `SEAD/CDE-09516` em
  [ensaio anterior](2026-11-04-varredura-unb-go.md), não selecionado
  por tentativa aleatória.
- Saídas completas mantidas em arquivos locais do runner; o log publicou
  apenas código de saída e campos saneados. Nenhum comando falho foi
  repetido. Não houve execução de JavaScript nem escrita de atribuição.

| Etapa | Resultado | Saída | Requisições observadas no contador da CLI |
| --- | --- | ---: | ---: |
| Build release | **PASS** | 0 | não aplicável |
| Login GO | **PASS** | 0 | 6 |
| Prévia de `--remover` GO | **PASS** | 0 | 5 |
| `--confirmar` / releitura de escrita | **NOT_RUN** | — | 0 |
| Remoção efetiva / releitura | **NOT_RUN** | — | 0 |

A prévia publicou `unidade=SEAD/CDE-09516`,
`confirmado=false`, `alterado=false`, `requisicoes=5`,
`subcodigo=null`. Não houve POST **de atribuição**, confirmação,
remoção real, retry ou resíduo criado/alterado pela operação.
As seis requisições de login são distintas das cinco da prévia;
**11 requisições** foram reportadas ao todo pela CLI nesta execução.
O êxito do job não deve ser usado como prova de escrita.

## Estado da capacidade

O critério de prévia em **duas instalações diferentes de treinamento**
agora possui evidência PASS: **UnB (4 GET, atribuição)** e
**GO (5 requisições, remoção)**. Em GO foi testada a prévia de
**remoção**, não a atribuição a um login escolhido; portanto esse
caminho específico ainda merece ensaio ao vivo, sem extrapolação.
As duas prévias UFRJ continuam **FAIL** com requisições não
observadas e causa não confirmada; PEN1 continua com login **FAIL**
por rede, sem prévia. Nenhum ensaio falho foi repetido.

O `--confirmar` **público continua recusado antes de sessão e rede**.
A conclusão da issue exige habilitação segura e ensaio autorizado no
treinamento com um único POST de atribuição, releitura independente,
remoção única, nova releitura, contagem e registro do resíduo. Até lá,
a capacidade é **WIP** e o critério de escrita permanece **NOT_RUN**.

O workflow de ensaio foi removido do branch na sequência e não
deve integrar a `main`.
