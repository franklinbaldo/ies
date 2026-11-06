---
type: SEIObservation
title: Validação de processo atribuir — transporte sintético, live não executado
description: Estado NOT_RUN de atribuição, sem sessão remota nem escrita, com POST único comprovado apenas em testes sintéticos
---

# processo atribuir — 06/11/2026

PR #316, issue #264. O fluxo injetável deriva o payload observado no
formulário Web e testa a sequência de **um POST e uma releitura nova**,
sem retry. Em bancada sintética, cada confirmação custa 5 GET + 1 POST
(6 tentativas). Há cobertura de atribuição por login exato, remoção com
`null`, falha de transporte no POST com releitura conclusiva e inconclusiva.
A CLI pública preserva a trava de segurança de `--confirmar` **antes de
abrir sessão**. RED no [CI 42195656454](https://github.com/franklinbaldo/ies/actions/runs/42195656454):
**4 testes novos FAIL, 1.689 PASS e 22 ignored preexistentes**.

## Replay específico do acervo (offline)

O teste `processo_atribuir::acervo::replay_de_processo_atribuir_no_acervo`
usa o **leitor real** `ler_tela` com as respostas históricas de sete
instalações, depois de conferir SHA-1 dos sete ZIPs, SHA-256 de cada
cassette, `schema_version`, completude e origem autorizada. O CI
[42201836542](https://github.com/franklinbaldo/ies/actions/runs/42201836542)
classificou **42 respostas**: **28 formulários GET aceitos** (quatro em
cada instalação), **zero recusas**, e **14 respostas POST marcadas como
não aplicáveis à prova de escrita**. Não se inferiu atribuição aplicada
do retorno histórico do POST. O replay utilizou **zero requisições reais**.

| Ensaio | Instalação | Estado | GET | POST | Resíduo |
| --- | --- | --- | ---: | ---: | --- |
| Prévia Web | UnB treinamento | **NOT_RUN** | 0 | 0 | nenhum |
| Prévia Web | segunda instalação de treinamento | **NOT_RUN** | 0 | 0 | nenhum |
| Confirmação única e releitura | treinamento | **NOT_RUN** | 0 | 0 | nenhum |
| Remoção única e releitura | treinamento | **NOT_RUN** | 0 | 0 | nenhum |

Não houve autenticação, requisição, escrita, retry nem consulta a produção.
A falta de ensaio live **não equivale a PASS**. A capacidade continua incompleta: faltam replay específico do acervo,
validação em duas instalações de treinamento e habilitação segura da
CLI pública. Nenhuma evidência sintética equivale a autorização para
escrita sem prévia ao vivo, confirmação única e releitura com registro
do resíduo.
