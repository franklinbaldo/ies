---
type: SEIObservation
title: Continuação e reconciliação da criação de processos em treinamento
description: Testes da Coruja e registro da confirmação de efeito incerto na UnB; sem repetir a escrita
operation: processo.criar
date: 2026-11-05
status: fail
---

# Continuação e reconciliação da criação de processos — issue #245

As seções anteriores ao ensaio estão em [relatório original](2026-11-05-processo-criar.md).
O estado live mais recente é **efeito incerto, resíduo desconhecido**;
proibido repetir a confirmação antes de conciliação por leitura.

## Retomada 🦉 Coruja — proteção de hipóteses AJAX (05/11/2026)

| Prova / etapa | Status | Requisições reais ao SEI | Resíduo |
| --- | --- | ---: | --- |
| Testes sintéticos RED de ID/nome AJAX não publicáveis | NOT_RUN — `cargo`/`rustc` ausentes, DNS do sandbox indisponível | 0 | Nenhum |
| GREEN/compilação Rust local | NOT_RUN — ambiente sem compilador Rust | 0 | Nenhum |
| Gate canônico e replay no novo head da PR #283 | NOT_RUN — não apareceu workflow do novo SHA na consulta ao GitHub | 0 | Nenhum |
| Prévia autenticada UnB e outra instalação de treinamento | NOT_RUN — sem conectividade/sessão autenticada | 0 | Nenhum |
| Uma única confirmação `--confirmar` em treinamento | NOT_RUN — não houve prévia ao vivo | 0 | Nenhum |
| Releitura verificadora do NUP | NOT_RUN — nenhuma criação efetuada | 0 | Nenhum |

Os testes Rust foram **publicados antes da correção**, em commits
`c979189` (2 testes unitários) e `be2fb9c` (1 teste de transporte
com orçamento exato de 3 GET + 1 POST AJAX consultivo, zero POST
criador). O teste integrado exige saída tipada
`hipotese_legal_ajax_resposta_nao_reconhecida` quando o XML anuncia
uma URL como identificador. O código anterior selecionava tal valor
por unicidade ou por `ID#NOME`; a execução RED ainda não foi
observada — **não confundir dedução do código com RED executado**.

A implementação no head seguinte recusa ID/nome não publicável tanto
no caso de uma única opção quanto na escolha explícita. Há também
teste de recusa da escolha segura sobre uma lista contaminada. A
correção não introduz requisição nem escrita. Executar os gates e
o replay antes de integrar; os últimos CI verdes citados acima
pertencem a SHAs anteriores e não comprovam o estado atual.

**Total live nesta retomada: 0 requisições ao SEI, 0 POST criador,
0 processos criados/alterados e nenhum resíduo.** A autorização de
escrita permanece válida apenas em treinamento; a sequência
prévia → um único `--confirmar` → releitura ainda não aconteceu.

## Reconciliação obrigatória após confirmação incerta — UnB treinamento

**Evidência real de 05/11/2026 (UTC, 21h47–21h49):**
[workflow de ensaio 42115596616](https://github.com/franklinbaldo/ies/actions/runs/42115596616),
job `UnB treinamento - previa confirmacao unica e releitura`.
Essa observação é posterior às seções históricas NOT_RUN acima e as
**substitui** quanto ao estado live da issue. A origem conferida pelo
roteiro era exclusivamente `https://sei.treinamento.unb.br/sei`.

| Etapa | Resultado observado | Requisições do comando | Resíduo |
| --- | --- | ---: | --- |
| Login UnB treinamento | PASS, saída 0 | 6 | sessão privada transitória no runner |
| Prévia pública de `Teste: Nível de acesso só público` | PASS, saída 0, `confirmado: false` | 3 (3 GET, 0 POST criador) | nenhum processo criado na prévia |
| **Uma** chamada `--confirmar` | **FAIL / efeito incerto**, saída **3**, `efeito_incerto_na_criacao` | **não observado** na resposta de erro; POST criador possivelmente efetuado | **DESCONHECIDO** |
| Releitura independente do NUP | NOT_RUN: confirmação terminou com erro e não revelou NUP | 0 | **a verificar na caixa Gerados** |

O roteiro interrompeu a execução imediatamente após a confirmação
incerta, sem repetir a escrita nem tentar outro backend.
O valor de NUP **não foi observado**. Portanto, **não é correto
registrar resíduo zero nem alegar processo criado**. Antes de qualquer
nova confirmação deve-se reconciliar por **leitura somente** a
caixa Gerados da conta pública UnB e, quando possível, conferir
tipo/especificação `Ensaio IES 245 - escrita unica em treinamento`
contra o processo. Uma leitura sem correspondência não prova que
a escrita falhou.

**Mitigação:** removido o workflow temporário de disparo por push
no commit `44005cb2`; não há autorização para reexecutar a confirmação
incerta. Nova infraestrutura de teste offline/manual da mesma PR
possui marca durável antes do POST, impede reuso do mesmo registro,
faz a releitura independente comparando o JSON `numero` com o NUP
e recusa origens fora das instalações de treinamento. Não foi
usada para repetir a criação real.

**Contagem real conhecida:** 6 login + 3 prévia + 0 releitura = 9
requisições observadas; quantidade interna da confirmação incerta
**não observada**, portanto total HTTP global **desconhecido**.
Nenhuma produção foi acessada.
