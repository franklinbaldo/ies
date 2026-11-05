---
type: SEIObservation
title: Replay offline da criação de processos na issue 245
description: Conferência das cassettes de sete instalações pelos leitores Rust de criar processo
---

# Processo criar — replay do acervo público

**PASS do teste de replay** em 05/11/2026. [Workflow temporário de ensaio offline](https://github.com/franklinbaldo/ies/actions/runs/42102035089): 15 cassettes originais verificadas, 83 respostas classificadas. Rust 1.99.1. Os sete ZIPs públicos foram baixados do Internet Archive, todos passaram SHA-1; o leitor do acervo conferiu também SHA-256 do membro, esquema, completude e instalação. Nenhum pedido foi enviado ao SEI (0 requisições). O workflow foi temporário, não faz parte do contrato da CLI.

Com `IES_ACERVO_CACHE` contendo os sete ZIPs originais, repetir:

```sh
scripts/replay processo_criar_fluxo::acervo::replay_de_processo_criar_no_acervo
```

O teste em `src/processo_criar/acervo.rs` percorre os cards `processo-criar-*` e usa **os leitores reais** da lista de tipos, do cadastro e das hipóteses AJAX. O arquivo da cassette não é tratado como uma ação a repetir: são lidas somente respostas já gravadas.

| Leitor | Respostas | Lidas | Recusadas | Não aplicáveis |
|---|---:|---:|---:|---:|
| Lista de tipos | 37 | 36 | 1 | 0 |
| Cadastro | 23 | 23 | 0 | 0 |
| Hipótese legal por AJAX | 2 | 0 | 2 | 0 |
| POST de criação (sem correlação de releitura neste replay) | 8 | 0 | 0 | 8 |
| Outras respostas não reconhecidas pelo leitor | 13 | 0 | 0 | 13 |
| **Total** | **83** | **59** | **3** | **21** |

A única recusa de lista ocorreu em **Goiás**, `tipo_de_processo_ambiguo`: o replay não escolhe silenciosamente a primeira ocorrência. As duas respostas AJAX em **UnB** produziram `hipotese_legal_ajax_nao_unica`, porque o leitor exige uma única opção efetiva; esta recusa é **segura**, mas indica que a criação restrita na forma gravada exige seleção explícita que a CLI atual não oferece. Investigar a semântica da lista ao vivo antes de alterar o contrato; não inventar uma hipótese padrão.

Distribuição das listas lidas: UnB 7/7, UFRJ 5/5, Goiás 2/3, PEN1 7/7, PEN2 5/5, PEN3 5/5, PEN4 5/5. Cadastros: UnB 5/5, UFRJ 3/3, Goiás 1/1, PEN1 5/5, PEN2 3/3, PEN3 3/3, PEN4 3/3.

**Limite:** os corpos dos POSTs nem sempre estão preservados e este teste não estabelece identidade de um NUP criado a partir de uma requisição. Por isso os oito POSTs não são contabilizados como criações verificadas. O replay offline **não substitui** a prévia autenticada, confirmação única e leitura posterior ao vivo exigidas pela issue #245. A validação live continua NOT_RUN (0 requisições, nenhum resíduo).
