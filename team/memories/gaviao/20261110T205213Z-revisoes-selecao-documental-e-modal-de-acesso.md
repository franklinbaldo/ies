---
type: AgentMemory
title: "Seleção multicaixa em bloco e revalidação de acesso por modal"
description: "Revisões dos heads #701 e #698: bug funcional de associação checkbox↔documento na inclusão; modal sintética com contraprovas e CI oficial pendente."
agent: gaviao
timestamp: "2026-11-10T20:52:13Z"
tags: [revisao, bdd, bloco, validacao-acesso, anti-ies, sqlite, cassettes, ci, cafezinho]
---

# Para retomar

**Fato #701:** [parecer no head `a955483`](https://github.com/franklinbaldo/ies/pull/701#issuecomment-6778887892), **AJUSTES NECESSÁRIOS** apesar do [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42897607950). `src/bloco_incluir.rs` verifica DOC entre todos os títulos, mas a confirmação fixa `chkDocumentosItem0` e preserva `hdnDocumentosItensSelecionados`; em tela com duas caixas, pode escolher o ID errado. O HTML de exemplo do Todos tem checkbox0=50950/0011586 e checkbox1=50951/0011587, com selecionados=50950. Pedir 0011587 não pode enviar ID 50951 sob o nome da caixa0. Corrigir associação nome/ID/hidden em IES e anti-IES, contraprova multicaixa e segunda CLI no mesmo SQLite. O E2E atual de uma caixa **passou sinteticamente**, mas não falsifica esta situação.

**Fato #698:** [parecer head `479a044`](https://github.com/franklinbaldo/ies/pull/698#issuecomment-6778958944), **APTA AO CI COMPLETO**, [CI rápido verde](https://github.com/franklinbaldo/ies/actions/runs/42898667755), [CI oficial em execução](https://github.com/franklinbaldo/ies/actions/runs/42898779235) na última leitura (6/7 jobs concluídos, `prek` pendente). O diff mostra formulário hipotético da modal, POST único, releitura independente após 200/302, recusa sem retry e caminho com link guardado corrigido; quatro testes de CLI em `tests/aies_processo_parcial_cli/bdd_633.rs`, ainda sem reivindicar GREEN do CI oficial. Na comparação `main...head`, `behind=2`; sincronizar por merge verdadeiro e revalidar SHA. Checar se PR #700 duplicada foi efetivamente encerrada, sem tocar em seu branch.

**Evidências:** UnB bloco: card 152 interações e GET→POST302→GET, hashes/estrutura aprovados em [Actions #42889943882](https://github.com/franklinbaldo/ies/actions/runs/42889943882), mas corpo do POST não capturado. GO processo: cards existentes não cobrem modal. **Confrontos semânticos bilaterais NOT_RUN; treinamento/live NOT_RUN**, não impedem implementação sintética.

**Cafezinho:** visitei [#388](https://github.com/franklinbaldo/ies/issues/388). A reflexão da 🦊 sobre um HTTP 302 não contar toda a história aplica-se aos dois casos: é a associação específica e a releitura do estado que tornam o desfecho verificável, não apenas o carimbo da resposta. Isso é analogia, não teste.

**Próxima rodada:** revisitar os heads e novo CI; não repetir parecer no mesmo SHA; verificar correção da multicaixa e conciliação da main. Sem execução local de Rust, implementação funcional, merge, credenciais, produção ou acesso ao SEI nesta rodada.
