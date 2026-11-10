---
type: CommandContract
title: Consultar, prever e confirmar anotação de processo na unidade
operation: processo.anotar
command: ies processo anotar NUP [--texto TEXTO] [--confirmar]
effect: read-or-preview-or-confirmed-write
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

## Confirmação (BDD-ANOT-02)

Com `--confirmar`, somente instalações de treinamento explicitamente permitidas recebem a escrita. A CLI abre o processo, confere a unidade ativa e a anotação anterior no formulário original; deriva os campos e a URL assinada do GET servido, jamais de um NUP reconstruído. É efetuado **um único POST** com `txaDescricao=T`, sem seguir redirecionamento e sem retry. Uma releitura independente do formulário para a mesma identidade deve encontrar exatamente `T`; a resposta mutadora também precisa ter o 302 esperado, pois texto presente após recusa não prova que este POST o escreveu. Sem as duas provas, o comando encerra com código 3 e `codigo=efeito_incerto`, sem prometer que a escrita não ocorreu. Havendo texto preexistente, a gravação é recusada nesta fatia em vez de sobrescrever sem contrato para atualização.

JSON de sucesso: mesmos campos da prévia, agora `confirmado:true` e `alterado:true`. O SQLite compartilhado pelo anti-IES armazena o texto e o serve na próxima leitura; não há alteração de schema porque `anotacao_origem` já é textual. O teste vertical `bdd_anot_02_confirma_uma_vez_e_rele_retorno_da_anotacao` em `tests/aies_anotacao_cli.rs` é sintético, não certifica a resposta do SEI.

## Consulta (BDD-ANOT-03)

Sem `--texto`, `ies processo anotar NUP` consulta a anotação **sem enviar POST**, reutilizando o link de `anotacao_registrar` da barra e a conferência de `hdnIdProtocolo` e `txaDescricao` do mesmo processo. A saída humana mostra o texto publicado ou informa que não há anotação, sem alterar estado. Em `--json`, publica `anotacao` com o texto observado e `alterado:false`; apenas o formulário legível com textarea explicitamente vazia autoriza `anotacao:null`. Formulário ausente, ilegível ou de identidade divergente deve recusar a consulta em vez de inventar vazio. `--confirmar` exige `--texto`, e a consulta nunca dispara escrita.

A prova sintética `bdd_anot_03_consulta_texto_publicado_sem_post` executa os modos humano e JSON pela CLI Rust real, HTTPS loopback, anti-IES e SQLite; `bdd_anot_03_consulta_sem_anotacao_publica_null` verifica ausência observada. Os testes confirmam estado imutável, ausência de POST e orçamento observado. Comparação semântica independente com cassettes e ensaio de treinamento não estão demonstrados por esses testes.

**BDD-ANOT-04/05:** retirada e interação com envio permanecem pendentes; não ampliar esses cenários nesta PR.

[Decisão](../../docs/decisoes/processo-anotar.md) · [Issue #638](https://github.com/franklinbaldo/ies/issues/638)
