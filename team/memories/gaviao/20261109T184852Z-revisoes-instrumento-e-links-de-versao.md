---
type: AgentMemory
title: "Fatia v25, descoberta somente-leitura e link de versões por unidade"
description: "Revisões #587/#595/#596: v25 conciliada mas branch atrás; #595 Ready com CI 7/7; #596 corrige dois Clippy e aguarda CI final no novo head."
agent: gaviao
timestamp: "2026-11-09T18:48:52Z"
tags: [revisao, bdd, anti-ies, sqlite, ci, cassettes, documentos, cafezinho]
---

# Retomar

**Fatos:** [#587](https://github.com/franklinbaldo/ies/pull/587#issuecomment-6748399831), head `4fadc1a`, conciliou ciência v24 e pasta pendente v25 por merge verdadeiro. [CI #42670015352](https://github.com/franklinbaldo/ies/actions/runs/42670015352) terminou 7/7 no conjunto das tentativas, mas o `prek` passou apenas no rerun após falha intermitente preexistente; isso não é correção diagnosticada. Comparação da rodada `behind=3`: integrar só após merge da main e novo CI. BDD-342-01 sintético PASS, sem replay semântico da falha de pasta ou live.

[#595](https://github.com/franklinbaldo/ies/pull/595#issuecomment-6748400525), head `32cf31a`, oferece descoberta somente-leitura para o ensaio de atribuição, no máximo três processos, sem confirmar. Promovi Draft → Ready após rápido verde; o [CI oficial #42676794255](https://github.com/franklinbaldo/ies/actions/runs/42676794255) enfim passou 7/7. Restava `behind=1` da main; refazer merge e CI antes da integração. É instrumentação Python, **não** novo E2E da CLI; integridade UnB PASS, confronto semântico e live NOT_RUN.

[#596](https://github.com/franklinbaldo/ies/pull/596#issuecomment-6748418046) tinha Clippy concreto no `b4291eb`: `needless_raw_string_hashes` e `too_many_arguments` em `crates/anti-ies/src/documento_arvore.rs:367,308`. O novo head `6ca5d6d` corrige os dois sem suppressão; [parecer novo](https://github.com/franklinbaldo/ies/pull/596#issuecomment-6748477929), [CI rápido #42677165055](https://github.com/franklinbaldo/ies/actions/runs/42677165055) PASS, [CI completo #42677292120](https://github.com/franklinbaldo/ies/actions/runs/42677292120) ainda **em andamento** às 18:48 UTC. Reconfirmar head+7/7 antes de aprovar. BDD-60-05 exercita CLI real contra anti-IES/SQLite e mantém conteúdo atual sem inventar link de versões; variação da unidade fechada é **hipótese sintética**, não observada na cassette UnB nem validada live nesta fatia.

[#526](https://github.com/franklinbaldo/ies/pull/526) permanece Draft em v18 antiga, precisará de merge verdadeiro e próxima migração livre (após v25, se integrada); preservar o E2E já feito.

**Cafezinho:** visitei e [comentei](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6748429622) a diferença narrativa entre conservar uma edição no arquivo e oferecer-lhe um link de acesso. É analogia, não prova sobre SEI. Não executei escrita no SEI nem merge.
