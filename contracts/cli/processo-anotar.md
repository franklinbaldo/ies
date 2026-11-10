---
type: CommandContract
title: Prévia de anotação de processo na unidade
operation: processo.anotar
command: ies processo anotar NUP --texto TEXTO
effect: read-only-preview
implementation: partial
---

# Anotação de processo na unidade (#638)

**BDD-ANOT-01:** dado processo aberto na unidade da sessão, com formulário de anotação publicado e anotação vazia observada, `ies processo anotar NUP --texto "T"` devolve prévia da anotação pretendida e **não executa POST**. O texto anterior só pode ser publicado se o GET do formulário `frmAnotacaoCadastro` mostrar `hdnIdProtocolo` igual ao processo e `txaDescricao` único. Texto vazio do formulário é evidência de anotação vazia; link ou formulário ausente é observação **desconhecida**, não vazia.

Entrada NUP inválido e pedido de texto vazio saem com 2 antes de ler sessão. O fluxo abre o processo, lê a unidade efetiva e sua situação `aberto_em`, segue exclusivamente o link assinado `anotacao_registrar` publicado na barra do nó `PROCESSO`, confere a identidade e lê o conteúdo do formulário. Se a unidade não tem o processo aberto, o link não foi publicado ou a tela redirecionou para login, a prévia recusa sem POST, sem inferir estado. Nunca montar URL assinada a partir do NUP nem seguir redirecionamento.

A saída JSON tem `nup`, `id_procedimento`, `unidade`, `texto` pedido, `anotacao_antes` observado, `confirmado:false`, `alterado:false`, `origem:"web"` e `requisicoes`. A saída humana avisa que nenhuma escrita ocorreu. Nenhum segredo, cookie ou hash no resultado.

## Provas independentes

- **Integração sintética:** `tests/aies_anotacao_cli.rs`, CLI real → HTTPS loopback → anti-IES → SQLite, anotação vazia persistida e zero POST; estado apenas simulado, não prova comportamento real.
- **Cassettes:** `processo-escritas-unb` (37 interações) e `processo-escritas-go` (40 interações), ambas incluem `anotacao_registrar`. Integridade verificada no GitHub Actions, mas o confronto semântico completo com a tela/ação ainda é `NOT_RUN`.
- **Treinamento/live:** `NOT_RUN`. Nenhuma requisição ao SEI foi executada por este BDD.

**BDD-ANOT-02/03/04/05:** registro confirmado, leitura direta, retirada e interação com envio permanecem pendentes. Esta PR não aceita `--confirmar`; a capacidade está propositalmente limitada à prévia.

[Decisão](../../docs/decisoes/processo-anotar.md) · [Issue #638](https://github.com/franklinbaldo/ies/issues/638)
