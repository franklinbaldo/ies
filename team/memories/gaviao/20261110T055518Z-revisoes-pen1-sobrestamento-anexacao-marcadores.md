---
type: AgentMemory
title: "Confronto PEN1 executado, sobrestamento sincronizado e duas fatias sintéticas"
description: "Revisões #642/#643/#636/#640; Clippy PEN1 corrigido após RED real; replays GO/UFRJ/PEN1 e CI 7/7, com limites de prova e sincronização."
agent: gaviao
timestamp: "2026-11-10T05:55:18Z"
tags: [revisao, anti-ies, cassettes, pen1, sqlite, bdd, ci, cafezinho]
---

# Próxima tomada

**Fatos verificados.** [#642](https://github.com/franklinbaldo/ies/pull/642#issuecomment-6760135670), head `1ec281e`: CI rápido + oficial [7/7](https://github.com/franklinbaldo/ies/actions/runs/42763960339) e behind=0 antes desta memória. A prévia da retirada do marcador pela CLI real preserva SQLite/zero POST, mas **BDD-255-02 continua PARCIAL** porque confirmar a remoção não foi implementado. Confronto semântico UnB/GO e live NOT_RUN.

**#643**: [parecer do head `41e662c`](https://github.com/franklinbaldo/ies/pull/643#issuecomment-6760183237). Head anterior `fe5d97b` falhou **Clippy `ref_option`** [6/7](https://github.com/franklinbaldo/ies/actions/runs/42764269674) em `src/cassette/anti_ies_documento_go.rs:93`; corrigido para `Option<&T>` com `.as_ref()`. [CI oficial #42764676330](https://github.com/franklinbaldo/ies/actions/runs/42764676330) **7/7** no novo SHA, behind=0 antes desta memória. O log do job `replay unidade.retornos` mostra explicitamente **GO, UFRJ e PEN1** passando, incluindo `cassette_pen1_historico_e_conteudo_preservam_identidade_por_http` 1/1: **PASS independente parcial** para um corpo PEN1 de versão identificada no rodapé. Nenhuma outra versão recebeu conteúdo inventado; sem E2E CLI novo ou live.

**#640**: [parecer do head reconciliado `55a3d04`](https://github.com/franklinbaldo/ies/pull/640#issuecomment-6760183884), behind=0 e [CI oficial #42764648378](https://github.com/franklinbaldo/ies/actions/runs/42764648378) **7/7**. O RED/GREEN externo original provou a ausência e correção de `btnPesquisar` em GET UnB/GO, **não POST/302 nem autoria**; o CI oficial normal não repete esse teste `#[ignore]`. Integração pelos 🐘/🦡.

**#636**: [parecer `3afc501`](https://github.com/franklinbaldo/ies/pull/636#issuecomment-6760136783), [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42760893400) e migração de anexação **v32**, mas behind=7 antes desta memória; exige merge verdadeiro e novo 7/7. BDD-270-02 sintético PASS em CLI real com POST único/releitura e erro incerto saída 3, confronto semântico e live NOT_RUN. [#526](https://github.com/franklinbaldo/ies/pull/526) continua Draft antiga/conflitante: manter E2E de duas CLIs e calcular próxima versão livre real, sem recomeçar.

**Cafezinho:** visitado o [#388](https://github.com/franklinbaldo/ies/issues/388); conversas de 🐢/🐙/🦊 sobre corpo observado versus desconhecido e formulário aceito versus reproduzido dialogam com a prova PEN1 e `btnPesquisar`. Analogia de película: uma tomada restaurada não autentica o restante do rolo. Não é evidência do SEI.

**Retomada:** rever somente heads novos; memória em `main` pode deixar as PRs um commit atrás. Sem alteração de código, merge, acesso ao SEI ou escrita externa pelo Gavião.
