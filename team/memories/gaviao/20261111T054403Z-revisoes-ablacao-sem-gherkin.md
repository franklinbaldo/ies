---
type: AgentMemory
title: "Quatro revisões pós-Gherkin: duas aptas e duas para sincronizar"
description: "Heads #755/#750 tecnicamente aptos com CI 7/7 e main sincronizada; #757/#756 com RED/GREEN técnico 7/7, mas defasados da main. Cassettes não foram confrontadas semanticamente."
agent: gaviao
timestamp: "2026-11-11T05:44:03Z"
tags: [revisao, ablação, cassette-first, rust, anti-ies, ci, cafezinho]
---

# Ponto de retomada

- **#755** [parecer](https://github.com/franklinbaldo/ies/pull/755#issuecomment-6788138006), head `bee2061`: **PRONTA PARA INTEGRAR** no snapshot `behind=0`, CI oficial [7/7](https://github.com/franklinbaldo/ies/actions/runs/42966985129). Correção usa variantes já existentes para recusar atributo `value` repetido em linha/filtro; teste Rust sintético, sem novo E2E desta anomalia. Corrigida colisão de diretório efêmero da bancada com contador atômico. Inspeção anterior de três cassettes UnB/UFRJ/GO validou integridade, **não confronto semântico**.
- **#750** [parecer](https://github.com/franklinbaldo/ies/pull/750#issuecomment-6788138396), head `04ebd4c`: **PRONTA PARA INTEGRAR** no snapshot `behind=0`, CI oficial [7/7](https://github.com/franklinbaldo/ies/actions/runs/42966214090). Duas supressões eliminadas via `TelaServida` existente; quatro arquivos, sem wrapper. Cards UnB de prazo (33) e atribuição (24) lidos, replay bilateral novo **NOT_RUN**.
- **#757** [parecer](https://github.com/franklinbaldo/ies/pull/757#issuecomment-6788138783), head `09264e2`: LGTM ablação `NotADirectory` em erro local, RED/GREEN e CI [7/7](https://github.com/franklinbaldo/ies/actions/runs/42964422554), **behind=9**. Precisa merge verdadeiro e CI do novo SHA. Cassette SEI não aplicável.
- **#756** [parecer](https://github.com/franklinbaldo/ies/pull/756#issuecomment-6788139196), head `084efa6`: LGTM oito anotações `#[source]` com teste tipado, RED/GREEN e CI [7/7](https://github.com/franklinbaldo/ies/actions/runs/42963326094), **behind=11**. Precisa merge verdadeiro e CI novo. Cards GO/UFRJ de cancelar: inspeção de integridade prévia em 150 HTTP; sem replay semântico bilateral.
- **Método novo do proprietário:** desde 11/11, **não exigir nem escrever Gherkin/BDD/Cucumber**. Prioridade da fila #716–#723 é ablação comprovada; E2E com CLI real e respostas originais se mudança Web, mas hipótese sintética é rotulada e não veta integração quando live/REA não existem. Três níveis independentes nunca colapsam.
- **Cafezinho #388:** li a conversa recente da 🐢 sobre o RED/GREEN da #755 e da 🦫 sobre preservação de `ErrorKind` em #757. Não comentei.
- **#759** foi integrada à main durante a revisão e alterou muitos arquivos/contratos; ao revalidar #756/#757 observar composição com os gates atuais. Nenhuma operação em SEI externo, produção, credenciais, merge ou branch alheia nesta rodada.

Conferir heads e sincronização novamente na próxima rodada; não duplicar parecer do mesmo SHA.
