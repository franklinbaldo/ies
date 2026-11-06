---
type: SEIObservation
title: Validação de processo atribuir — transporte sintético e prévia UnB
description: Ensaio de prévia autenticada UnB sem escrita e transporte sintético com POST único
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
| Prévia Web | UnB treinamento | **PASS** | 4 | 0 | nenhum |
| Prévia Web | segunda instalação de treinamento | **NOT_RUN** | 0 | 0 | nenhum |
| Confirmação única e releitura | treinamento | **NOT_RUN** | 0 | 0 | nenhum |
| Remoção única e releitura | treinamento | **NOT_RUN** | 0 | 0 | nenhum |

**Ensaio UnB, 06/11/2026:** [GitHub Actions 42205702494](https://github.com/franklinbaldo/ies/actions/runs/42205702494), conta pública da instalação de treinamento. Login **PASS, saída 0, 5 requisições**; após intervalo superior a 30 segundos, prévia `processo atribuir 23106.001559/2026-11 --usuario aluno2` **PASS, saída 0, 4 GET e 0 POST de atribuição**. `confirmado=false` e `alterado=false`. Valores de cookie, credencial, URL assinada e `infra_hash` permaneceram em arquivos privados do executor; apenas código de saída e contagem foram publicados. A prévia NÃO comprova escrita. Não houve confirmação, remoção, retry de escrita, resíduo nem acesso a produção.

O ensaio em segunda instalação segue **NOT_RUN**, assim como a confirmação única e a remoção, porque `--confirmar` permanece fechado antes da rede. A capacidade continua incompleta: faltam ensaio na segunda instalação, confirmação e remoção únicas em treinamento com prova por releitura, resíduo registrado e habilitação segura da CLI pública.

## Regressão de destino 302 genérico

Observação de segurança: `acao=principal` não identifica, por si só,
o processo atribuído. A leitura coincidente após esse 302 pode ter sido
produzida por outro agente. O teste sintético foi introduzido antes
da correção, e o [CI 42204843683](https://github.com/franklinbaldo/ies/actions/runs/42204843683)
registrou **1 FAIL novo, 1.706 PASS e 23 ignored preexistentes**:
o fluxo marcava `confirmado=true` com 4 GET de prévia, 1 POST e 1 GET de
verificação, embora a resposta não identificasse o processo. A correção
exige `id_procedimento` explícito em redirects para `principal`;
rejeição continua com saída 3, sem segundo POST. Um destino `principal`
com ID correto permanece aceito e é testado; sem tráfego real nesta etapa.

## Regressão de destino `arvore_visualizar` genérico

A revisão 🦅 do head `0fc1922` identificou que
`acao=arvore_visualizar` sem `id_procedimento` ainda era aceito.
Um segundo agente poderia produzir a atribuição ou remoção desejada
entre o POST e o GET de verificação, causando falso sucesso.
O RED [CI 42214490209](https://github.com/franklinbaldo/ies/actions/runs/42214490209)
comprovou o defeito em **dois testes novos** (atribuir/remover):
**2 FAIL, 1.716 PASS, 23 ignored existentes**, após **4 GET de
prévia + 1 POST + 1 GET de releitura** por cenário.
O fluxo agora exige ID explícito também para `arvore_visualizar`.
Os testes de sucesso usam a forma `ACEITE` observada no Todos @323eb01,
com ação, origem e ID do mesmo processo. Os destinos genéricos continuam
efeito incerto (saída 3), sem repetir POST ou seguir o 302.
**Live desta fatia: NOT_RUN**, 0 requisições reais e 0 resíduos.
A prévia autenticada da UnB registrada acima continua sendo a última
evidência ao vivo; a confirmação pública segue travada.
