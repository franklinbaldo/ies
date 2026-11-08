---
type: AgentMemory
title: "Revisões #486/#490/#487/#489 e ordem das migrações"
description: "Quatro heads receberam parecer; CI oficial 7/7 no SHA de cada um; integração condicionada à sequência v12 PDF, v13 cancelamento, v14 pesquisa e conciliação final."
agent: gaviao
timestamp: "2026-11-08T17:39:39Z"
tags: [revisao, bdd, anti-ies, sqlite, cassettes, ci]
---

# Retomar na próxima rodada

**Fato revisado:** publiquei pareceres para [#486](https://github.com/franklinbaldo/ies/pull/486#issuecomment-6724799824) `b26b2a8` e [#490](https://github.com/franklinbaldo/ies/pull/490#issuecomment-6724800154) `e3287b4` como prontas no head examinado, CI oficial 7/7. Para [#487](https://github.com/franklinbaldo/ies/pull/487#issuecomment-6724800433) `9235a2b` e [#489](https://github.com/franklinbaldo/ies/pull/489#issuecomment-6724800738) `6e26f21`, há prova sintética verde, mas ambas exigem conciliação de migrações na ordem **0012 PDF (#486) → 0013 cancelamento (#487) → 0014 pesquisa negativa (#489)**, merge verdadeiro e novo CI no SHA reconciliado. #490 cobre o comando literal BDD-333-05 sem `--manter-aberto` e o envio posterior no mesmo SQLite.

**Três provas distintas:** inspeções de integridade de cassettes foram executadas pelos autores, sem confronto semântico independente; nenhum PASS live/REA foi alegado. Não converter integridade em paridade externa, nem bloquear por ela. Os heads podem mudar após merge e só o novo CI serve ao novo SHA.

**Cafezinho:** a [discussão sobre identificabilidade e a pesquisa com zero resultados](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6724676126) reforça como inspiração (não evidência) que ausência de cena registrada não é cena comprovadamente vazia. Na revisão, cobrar declaração honesta de observação e estado desconhecido.
