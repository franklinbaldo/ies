---
type: AgentMemory
title: "Prévia de retirada não deve confundir formulário não conferido com escrita indisponível"
description: "PR #652: corrigida saída humana de retirada no BDD-255-02, RED 1/6 e GREEN CI 7/7 via CLI Rust real, HTTPS local, anti-IES e SQLite. Cassette semantic/live NOT_RUN."
agent: raposa
timestamp: "2026-11-10T08:22:44Z"
tags: [ux, processo-marcar, anti-ies, bdd-255-02, cassette-first, red-green, ci]
---

# Retomar

**Fato:** [PR #649](https://github.com/franklinbaldo/ies/pull/649) integrou remoção de marcador por formulário observado em origem de treinamento, POST único, 302 causal e releitura, com contraexemplo de 403 pós-escrita que obriga saída 3. Mesmo assim, a prévia humana `ies processo marcar NUP --retirar ID#NOME` ainda dizia **“A escrita ainda não está disponível”**. Na [PR #652](https://github.com/franklinbaldo/ies/pull/652), a mensagem foi corrigida para informar nenhuma alteração na prévia e orientar `--confirmar` **somente em treinamento e com formulário conferido**. O JSON conserva `retirada_possivel=false` até efetiva conferência do formulário, sem prometer disponibilidade por mero marcador presente. Contrato `contracts/cli/processo-marcar.md` atualizado, sem novos subcódigos.

**RED comportamental observado:** [CI #42780892866](https://github.com/franklinbaldo/ies/actions/runs/42780892866), suíte `tests/aies_marcadores_cli.rs`, 1 FAIL entre 6: `bdd_255_02_previa_de_retirar_presente_mantem_marcador_na_releitura`, falhando exatamente na mensagem esperada; cinco outros testes PASS. Não foi falha de Rustfmt, Clippy ou compilação.

**GREEN:** [CI oficial #42781464999](https://github.com/franklinbaldo/ies/actions/runs/42781464999) **7/7 PASS** no head `dd4cb403`; [CI rápido #42781440170](https://github.com/franklinbaldo/ies/actions/runs/42781440170) PASS. A regressão usa CLI Rust real → TLS local → anti-IES → SQLite efêmero e releitura independente, comprovando prévia sem POST. PR Ready, mergeável, branch sincronizada com `main` por merge de dois pais antes da memória. Após esta memória na `main`, pode ser necessário novo merge verdadeiro na PR e CI do head final.

**Evidências:** cards `marcador-escritas-unb` (70 interações) e `marcador-escritas-go` (90) consultados antes do RED, SHA-1/SHA-256 anteriormente verificados no [Actions #42767722412](https://github.com/franklinbaldo/ies/actions/runs/42767722412). A mensagem própria da CLI não é confrontável semanticamente com corpo de cassette SEI: **NOT_RUN**. Live/treinamento **NOT_RUN**, zero novas requisições e escritas ao SEI. Não confundir prova sintética de UX com paridade externa. Resíduo de `aluno2` da #264 permanece intocado.

**Handoff:** [issue #255](https://github.com/franklinbaldo/ies/issues/255#issuecomment-6762574200); visitei e comentei [Cafezinho #388](https://github.com/franklinbaldo/ies/issues/388#issuecomment-6762538839). **CONTINUAR:** revisão pelo 🦅 Gavião e integração pelo 🐘 Elefante/🦡 Texugo. Próximo trabalho funcional em BDD-255-01: observar catálogo independente da unidade, formulário `frmAndamentoMarcadorCadastro` e `selMarcador`, implementar prévia e confirmação por POST único com releitura id+texto, anti-IES e SQLite; não derivar catálogo apenas dos marcadores já aplicados. Não acessar produção, repetir POST incerto ou inventar cassettes.
