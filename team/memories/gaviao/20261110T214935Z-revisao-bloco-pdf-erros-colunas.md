---
type: AgentMemory
title: "Revisão de inclusão em bloco, PDF cruzado, contratos tipados e colunas"
description: "Quatro pareceres: #701 e #706 aprovadas na fatia sintética/técnica, #705 documental 7/7, #703 bloqueada por import Clippy; sincronização concorrente exige novos SHAs."
agent: gaviao
timestamp: "2026-11-10T21:49:35Z"
tags: [revisao, bdd, bloco, pdf, erros-tipados, clippy, cassettes, ci, cafezinho]
---

# Para retomar

- **#701, head `8cf6ba7`:** [parecer](https://github.com/franklinbaldo/ies/pull/701#issuecomment-6780151447), BDD-381-02 **LGTM sintético**, [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42907767286). Correção associa `chkDocumentosItemN` a ID e número SEI; regressão com duas caixas confirma só o alvo por segunda CLI no mesmo SQLite. RED comportamental não comprovado. UnB: hashes/inventário sim, corpo POST e confronto semântico bilateral não; live NOT_RUN.
- **#706, head examinado `9e6ee1`:** [parecer](https://github.com/franklinbaldo/ies/pull/706#issuecomment-6780152020); [CI 7/7](https://github.com/franklinbaldo/ies/actions/runs/42907610815), RED demonstrou 503 errado antes do 404 para PDF público comprovadamente de outro processo; teste novo é HTTP socket + SQLite, **não** CLI real desta variante. Dois cards UnB (75 interações) não registram negativo; live NOT_RUN. **A branch avançou em seguida para `8610a2a` com merge de `main` e CI do novo SHA pendente**: reavaliar sem repetir argumentos de código já observados.
- **#705, head `2e9541`:** [parecer documental](https://github.com/franklinbaldo/ies/pull/705#issuecomment-6780152625), CI [7/7](https://github.com/franklinbaldo/ies/actions/runs/42906728416), integrada à `main` logo depois. Sugestão: trocar a palavra “subcódigos próprios” ainda presente em `contracts/cli.md` por “variantes”; e atualizar referência ao BDD-381-02 após integrar #701.
- **#703, head `8b3084c`:** [parecer AJUSTES NECESSÁRIOS](https://github.com/franklinbaldo/ies/pull/703#issuecomment-6780153530). [CI 6/7](https://github.com/franklinbaldo/ies/actions/runs/42903516460), Clippy real acusa import não utilizado de `EstadoDasColunas` em `src/visao_do_controle.rs:35`; ainda Draft. BDD-565-01 CLI real ↔ anti-IES ↔ SQLite NOT_RUN, não atribuir PASS só à refatoração.
- **Cafezinho #388:** [conversa da 🦊 sobre a caixa documental](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6779678907) confirma a utilidade de verificar o vínculo da legenda ao ID na releitura. É reflexão, não prova de cassette.
- **Próxima rodada:** ler SHAs e CI atuais antes de novos pareceres; ignorar heads já revisados; revalidar sincronização após as memórias e integração concorrentes. Não houve código funcional, merge, credenciais, live nem acesso ao SEI pelo revisor.
