---
type: SEIObservation
title: "UnB treinamento: recusa tipada por unidade não observada na árvore do processo"
description: "Prévia única de atribuição em segundo NUP conhecido: login PASS, recusa tipada com comparação de unidades falsa e nenhuma confirmação de escrita"
---

# Processo atribuir — diagnóstico UnB de 09/11/2026

**BDD:** [#264](https://github.com/franklinbaldo/ies/issues/264), especialmente BDD-264-01.
**Execução:** [Actions #42662410727](https://github.com/franklinbaldo/ies/actions/runs/42662410727), commit `12bd7cc` de uma branch temporária derivada da `main`. O job completou com sucesso **como executor do ensaio**, não como aprovação da prévia.

## Observação efetiva

Somente a origem exata `https://sei.treinamento.unb.br/sei` foi aceita. A CLI Rust foi compilada em modo release com Rust 1.99.1. A conta pública de treinamento foi lida da tabela oficial; arquivos de credenciais, origem e sessão ficaram privados (`0600`), sem publicar senhas, cookies, hashes de sessão ou links assinados.

| Etapa | Resultado da CLI | Requisições observadas |
| --- | --- | ---: |
| Build da CLI real | PASS | Não aplicável |
| `ies --json login` único | PASS, `login_realizado`, saída 0 | 6 |
| Prévia `ies processo atribuir 23106.001559/2026-11 --usuario aluno2` única | **FAIL**, saída CLI 1, `situacao_da_atribuicao_nao_observada` | 4 |
| `--confirmar`, releitura pós-POST e remoção | **NOT_RUN** | 0 pedidos dessas etapas |

A nova instrumentação da [PR #577](https://github.com/franklinbaldo/ies/pull/577) publicou `unidade_consta_na_arvore: false`: a **unidade publicada pela tela ativa não consta na lista de unidades abertas publicada pela árvore** neste instante. As siglas não foram divulgadas. A comparação é um fato obtido de dois campos observados na recusa, não evidência de qual processo ou usuário poderia ser usado para escrever.

O executor devolveu `PREVIA_FAIL`, `saida_cli: 1`, `requisicoes: [4]`, `checkpoint: false`; portanto a guarda impediu a confirmação, a remoção e qualquer POST de atribuição. A autenticação tem seu próprio tráfego, contado separadamente. GETs da prévia podem ter efeitos de leitura/recebimento no SEI; não declarar ausência geral de efeitos.

## Comparação e limites

Esta foi uma **nova prévia de processo diferente** do ensaio inconclusivo de [09/11, #42625001594](https://github.com/franklinbaldo/ies/actions/runs/42625001594), cujo NUP foi `23106.001748/2026-94`. O NUP desta rodada já tivera prévia bem-sucedida em [06/11, #42205702494](https://github.com/franklinbaldo/ies/actions/runs/42205702494): isso **não garante** que a unidade e a situação permaneçam iguais dias depois numa conta de treinamento compartilhada. A nova comparação confirma a divergência de unidades **agora**, não a causa retrospectiva de 09/11 no outro processo.

- **IES real ↔ anti-IES ↔ SQLite:** PASS sintético anterior nos BDD-264-01/02/03; não reexecutado nesta verificação externa.
- **IES e anti-IES ↔ cassettes:** confrontos anteriores restritos a 28 formulários GET e 14 respostas POST históricas caracterizadas, sem prova causal de escrita.
- **SEI de treinamento:** login PASS; prévia FAIL com causa tipada observada; confirmação, remoção e verificação de escrita NOT_RUN, **sem resíduo de atribuição confirmado**.

**Segurança:** nenhum retry de prévia falha, nenhum `--confirmar`, nenhuma escrita em produção. O workflow de ensaio, criado apenas para esta observação, foi removido da branch no commit `ab4fbfc`.

**Próximo teste com justificativa:** localizar previamente, por observação independente e segura na instalação de treinamento, um processo aberto **na unidade ativa da sessão** e sem atribuição, em vez de repetir NUPs que já recusaram. Só executar um novo ensaio de confirmação/remoção após uma prévia realmente PASS, com POST único, releitura e registro do resíduo. Não inferir ausência de atribuição a partir de unidade não observada.
