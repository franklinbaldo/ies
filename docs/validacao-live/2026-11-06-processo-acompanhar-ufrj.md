---
type: OperationEvidence
title: Ensaio UFRJ de Acompanhamento Especial — confirmação única com efeito incerto
description: Segunda instalação de treinamento com prévia autenticada, uma tentativa de escrita e releitura independente, sem reenvio
operation: processo.acompanhar
---

# 🐢 UFRJ de treinamento — ensaio da issue #254, 06/11/2026

## Escopo e autorização

- **Instalação:** exclusivamente a UFRJ de **testes/treinamento**, origem conferida por igualdade literal com a tabela pública `docs/interfaces/credenciais-de-treinamento.md`. Nenhuma produção acessada.
- **Processo de treinamento:** `99079.901790/2026-43`, já observado por leitura independente na UFRJ no trabalho da #264. Unidade publicada nesta operação: `ADCENTRAL`.
- **Grupo pedido:** padrão vazio `""`; **observação enviada:** `IES254 Tartaruga UFRJ 2026-11-06 tentativa unica`.
- **Execução:** [run 42245344772](https://github.com/franklinbaldo/ies/actions/runs/42245344772), workflow temporário adicionado exclusivamente ao branch `codex/254-tartaruga-exclusao` e **removido após o ensaio**. O workflow foi executado em `ubuntu-22.04`, com Release compilado no CI.
- **Segurança:** credenciais publicadas pela própria instalação, mas lidas da tabela para arquivos locais privados `0600` em diretório `0700`; cookie, URL do Controle, senha e `infra_hash` não foram impressos. Saídas divulgadas contêm somente campos JSON selecionados, nunca o corpo HTTP ou URL assinada. Pausa de 35 segundos entre comandos; nenhum retry, backend alternativo ou execução de JavaScript.

## Sequência e resultado realmente observado

| Etapa | Resultado | Requisições publicadas | Evidência |
| --- | --- | ---: | --- |
| `ies login --json` | **PASS**, saída 0 | **6** | Sessão na origem exata de treinamento |
| Prévia `processo acompanhar NUP --grupo "" --observacao MARCA --json` | **PASS**, saída 0 | **6** | `operacao=acompanhar`, `ja_estava=false`, `confirmado=false`, `alterado=false`, `grupo=""`, `id_acompanhamento=null`, marca e NUP idênticos ao pedido |
| **Único** `--confirmar` após guarda e pausa | **FAIL / EFEITO INCERTO**, saída **3** | **8** | `codigo=efeito_incerto`, `subcodigo=acompanhamento_nao_confirmado_por_leitura`, `motivo=resposta_sem_aceite` |
| Releitura independente, sem `--confirmar` | **PASS**, saída 0 | **5** | `ja_estava=true`, **ID observado `142`**, grupo vazio, observação **exatamente igual** à marca enviada |

**Total contado das quatro invocações: 25 requisições** (6 + 6 + 8 + 5), incluindo autenticação. O backend da confirmação tem **uma única tentativa de POST de acompanhamento no máximo** e exige releitura; não há retry. O contador de 8 pertence à invocação confirmadora e foi publicado pela CLI, diferentemente da tentativa antiga na UnB, cujo contador não fora observado.

A guarda do executor só autorizou a confirmação depois de verificar NUP, unidade não vazia, grupo, observação, operação, estados e orçamento da prévia. **Não ocorreu uma segunda confirmação.**

## Resíduo e limite da conclusão

Na releitura posterior, apareceu o acompanhamento **ID 142** no processo `99079.901790/2026-43`, grupo `""`, com a marca `IES254 Tartaruga UFRJ 2026-11-06 tentativa unica`. Esse é o **resíduo observado**, que deve ser preservado. O estado não estava observado na prévia. Contudo, a conta é compartilhada e a confirmação não apresentou o aceite exigido; a observação posterior **não prova a autoria nem transforma a saída 3 em PASS**.

**Não repetir `--confirmar` neste NUP e não apagar automaticamente o ID 142**. Não houve tentativa de retirada; `--deixar --confirmar` público continua bloqueado **antes de sessão/rede**. Nenhuma escrita em produção.

## Validação, RED e pendências

- **Compilação Release do executor:** PASS no run acima. **Live:** login PASS 6, prévia PASS 6, confirmação FAIL/INCERTO 8, leitura independente PASS 5. **Segunda instalação foi efetivamente ensaiada**, mas sem confirmação inequívoca.
- **RED novo:** NOT_RUN — este ensaio não altera Rust nem testes. RED histórico do último bug corrigido: [42235474727](https://github.com/franklinbaldo/ies/actions/runs/42235474727), 1 FAIL novo / 1.746 PASS / 23 ignored preexistentes; GREEN [42236346779](https://github.com/franklinbaldo/ies/actions/runs/42236346779), 6/6 no head histórico `76b6d4e`. O novo head após incorporação de `main` e remoção do workflow exige CI próprio.
- **Investigação seguinte:** descobrir por evidência de resposta *saneada* qual forma de aceite a UFRJ devolveu ao POST (status e destino/forma sem URL assinada), sem repetir esta escrita. A observação de estado no processo com a marca permite hipótese, não conclusão causal; construir fixture sintética e RED antes de ampliar qualquer critério. Repetição de teste com mutação só em **outro processo de treinamento**, após prévia inequívoca e registro de resíduo.
- **Issue #254 permanece WIP**: retirada real segura com prévia → único `--confirmar` → releitura ainda não foi demonstrada; `--alterar` é opcional. O bloqueio público da retirada não deve ser levantado por este resultado.
