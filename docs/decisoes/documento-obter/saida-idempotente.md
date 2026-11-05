---
type: ImplementationDecision
title: Gravar só a foto DOC.vN.html, de forma idempotente, em documento DOC --saida
operation: documento.obter
implementation: partial
status: adopted-for-scope
scope: apêndice da decisão de documento.obter (#207); com --saida DIR, ies documento DOC grava só DOC.vN.html, o HTML servido da versão N lida, sem DOC.txt, DOC.vN.txt nem base e manifesto ocultos; mesmo SHA-256 é no-op, conteúdo diferente é recusa tipada; o SHA-256 vai para o índice local, que a prévia de documento salvar confere
selected_strategy: foto-por-versao-idempotente
alternatives: foto-por-versao-idempotente,texto-e-copia-de-trabalho,foto-com-manifesto-oculto,sobrescrever
evidence_level: testes-sinteticos-replay-do-acervo-e-live-unb
live_validation: pass-unb-4-0-9
---

# Foto `DOC.vN.html` idempotente (#207)

Apêndice da [decisão de `documento.obter`](../documento-obter.md);
contrato em [versões e foto](../../../contracts/cli/documento-obter-versoes.md).
Decisão do proprietário de 05/11/2026. **Substitui** o nome do texto
`DOC.vN.txt` de #204 e a cópia de trabalho de #88 (com
`.DOC.vN.base.html` e `.DOC.vN.meta.json`), descritos na
[decisão de versões](versoes.md).

## Alternativas consideradas

- **foto-por-versao-idempotente** (escolhida): um arquivo por versão,
  `DOC.vN.html`, com o HTML que o SEI serviu; o nome carrega a
  identidade (número SEI e versão) e o SHA-256 decide entre gravar,
  não fazer nada ou recusar.
- **texto-e-copia-de-trabalho:** o desenho anterior (#88, #204): texto
  em `DOC.txt` ou `DOC.vN.txt` e, só quando a barra declara edição não
  assinada, a cópia `DOC.vN.html` com base e manifesto ocultos.
- **foto-com-manifesto-oculto:** a foto mais um `.DOC.vN.meta.json`
  com o SHA-256 e o id interno.
- **sobrescrever:** gravar sempre por cima.

## Comparação

- **Princípio.** No SEI, uma versão fechada não muda. Com a versão no
  nome, ler duas vezes dá o mesmo arquivo: o no-op é a resposta certa,
  e um conteúdo diferente só pode ser foto alterada localmente ou
  versão que divergiu, os dois casos que pedem recusa.
- **Arquivos.** O desenho anterior deixava até quatro arquivos por
  versão (texto, cópia, base, manifesto) e recusava a segunda leitura
  (`arquivo_texto_documento_ja_existe`, `copia_versao_ja_existe`). O
  texto se obtém do HTML; a base é a própria foto; o id interno está no
  [índice local](../../arquitetura/0003-indice-local.md).
- **Onde guardar o SHA-256.** A prévia de `salvar` precisa saber se a
  foto vN ainda é o que o SEI serviu. Sem arquivo oculto, o lugar é o
  índice local (tabela `foto`, esquema 3), que já guarda o que foi
  observado na instalação. O manifesto oculto repetiria o que o índice
  já faz e voltaria a espalhar arquivos.
- **Sobrescrever** apagaria a edição local `DOC.v(N+1).html` não
  enviada se a numeração coincidisse e viola o "sem sobrescrever".
- **Custo.** Nenhuma requisição a mais: N vem do rodapé servido (G + 1);
  só sem rodapé a lista é lida (G + 2, o caso já medido em #88). Sem
  lista publicada, recusa tipada, sem gravar com nome incerto. A barra
  não é mais lida quando o rodapé basta.

## Decisão

Adotada **foto-por-versao-idempotente**:

- `--saida DIR` grava `DOC.vN.html` para qualquer versão lida (atual ou
  `--versao N` histórica, assinada ou não), em UTF-8, `0600`, por
  parcial criado novo no mesmo `DIR`, `fsync` e `hard_link` (atômico,
  nunca sobrescreve, não segue link).
- HTML com `infra_hash` é `conteudo_com_link_assinado` (credencial nunca
  vai a arquivo).
- Mesmo SHA-256: `arquivo.gravado: false`, `motivo: igual`, saída 0.
  Diferente: `foto_local_divergente`. O lote inteiro é conferido antes
  da primeira gravação.
- Antes da rede: `DIR` seguro sempre; com `--versao N`, cada
  `DOC.vN.html` presente tem de ser legível, privado e coerente com o
  nome (título do mesmo número SEI e rodapé, se houver, da versão N).
  Os bytes só se comparam depois da leitura.
- `--sem-copia` saiu (não há mais cópia opcional).
- **Externo:** continua `DOC.pdf`, `DOC.txt` ou `DOC.html` (o arquivo
  original), sem sobrescrever, como na fatia 3 do #60. O externo não
  serve rodapé nem publica a lista (observado), então não há versão
  para o nome; `--versao N` é recusado antes do anexo. Torná-lo
  idempotente pelo SHA-256 do anexo é outra decisão. Só o objeto
  `arquivo` do JSON mudou de forma, igual ao do interno.
- `--html-privado ... --saida` (offline) segue gravando `DOC.txt`: não
  há versão servida.

## Evidências verificáveis

- Testes sintéticos: `foto_documento::tests`,
  `processo_obter::tests_documento_versoes`, `documento_versao_cli::testes`,
  `documento_obter_cli::testes`, `copia_documento::tests_par`,
  `copia_listagem::testes`, `indice_local::tests_fotos` e
  [cli_documento_saida](../../../tests/cli_documento_saida.rs).
- RED: 26 testes falhando com a implementação em stub (23 unitários,
  3 de CLI); um deles era o teste de migração 1 → 2, que precisou
  remover também a tabela nova.
- Replay do acervo (`scripts/replay`) de `replay_de_documento_no_acervo`
  e `replay_de_versoes_no_acervo`: relatório idêntico ao de `main`.
- [Ensaio live na UnB 4.0.9](../../validacao-live/2026-11-05-documento-saida-idempotente.md):
  leitura repetida sem gravação, mesmo SHA-256.

## Limitações e revisão

A idempotência supõe que o SEI serve os mesmos bytes para a mesma
versão: visto na UnB (duas leituras), não nas outras instalações. Se
uma instalação variar o HTML (data, token), a segunda leitura sairia
`foto_local_divergente`: rever então para comparar só o conteúdo do
corpo. Sem índice (`IES_SEM_INDICE` ou banco desligado), a foto é
gravada mas não registrada, e a prévia de `salvar` recusa a base. Um
parcial `.DOC.vN.html.parcial-PID` pode sobrar se o processo morrer
entre a criação e a remoção. A escrita remota (#89) fica fora.
