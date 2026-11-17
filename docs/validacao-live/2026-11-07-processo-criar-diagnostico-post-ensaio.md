---
type: SEIObservation
title: Preservação de causa e possível resíduo no ensaio pós-POST
description: Regressão observada no roteiro da issue 245, RED/GREEN offline e limites da validação live
operation: processo.criar
date: 2026-11-07
status: not-run
---

# 🦉 #245 — diagnóstico do POST inconclusivo no roteiro de ensaio

## Observação e defeito

A fronteira Rust `src/processo_criar_cli.rs::erro_do_fluxo` publica
`subcodigo`, `causa_post` e, quando há coincidências após leitura,
`candidatos_compativeis` e `candidatos_observados_na_caixa`.
São fatos sobre a tentativa, **não** prova de autoria.

O roteiro `scripts/ensaio_processo_criar.py` descartava todos esses
campos quando `--confirmar` retornava saída 3, gravando somente
`EFEITO_INCERTO` e a contagem. Com isso, um possível NUP de resíduo
identificado pela própria CLI não chegava ao registro privado persistente
do ensaio, dificultando a conciliação sem nova escrita.

## RED → GREEN

- Teste primeiro no commit `a1f7f4a`: saída sintética da CLI com
  `candidato_unico_sem_prova_causal`, `causa_post: falha_de_rede`,
  NUP compatível e 8 requisições informadas.
- [CI RED 42363897352](https://github.com/franklinbaldo/ies/actions/runs/42363897352):
  **24 testes Python executados, 1 ERROR novo** (`KeyError: 'subcodigo_post'`),
  pois o ensaio anterior perdia a causa. Nenhuma rede SEI no teste.
- A implementação preserva `subcodigo_post`, `causa_post`,
  `causa_releitura`, status HTTP quando presentes, e NUPs compatíveis
  ou observados sob `possiveis_residuos`. Identificadores são
  saneados e nunca elevam candidato a criação confirmada.
- O teste de privacidade rejeita strings contendo URLs assinadas
  nos campos de causa ou NUP.
- Nenhuma requisição adicional, POST, retry, reconciliação causal nova
  ou mudança da CLI Rust. A trava local de execução permanece 0600
  e não é removida após falha.

## Validação live

| Etapa | Estado | Requisições HTTP reais |
| --- | --- | ---: |
| RED/GREEN sintético offline | Executado no CI; sem SEI | 0 |
| Prévia autenticada em segunda instalação de treinamento | NOT_RUN | 0 |
| Confirmação única em treinamento | NOT_RUN | 0 |
| Leitura independente real | NOT_RUN | 0 |
| Escrita e resíduos novos nesta fatia | NÃO EXECUTADOS | 0 |

O acesso autenticado seguro a uma **segunda** instalação de treinamento
continua indisponível neste ambiente. Não houve acesso a produção ou
repetição da criação UnB historicamente incerta. A saída 3 não autoriza
reexecutar `--confirmar`; NUP coincidente não prova efeito causal.

A issue #245 permanece aberta até o ciclo live
prévia → confirmação única → verificação independente, com contagem
real e registro do resíduo.
