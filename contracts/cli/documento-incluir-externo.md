---
type: CommandContract
title: Inclusão de documento externo PDF — inspeção local e prévia Web antes da escrita
operation: documento.incluir-externo
command: ies documento incluir-externo --processo NUP --arquivo ARQUIVO.pdf --tipo TIPO [--consultar-sei] [--confirmar]
effect: write-document-external
implementation: partial
---

# Incluir documento externo em processo — #332

A capacidade está **PARCIAL**. Por padrão, `ies documento incluir-externo --processo NUP --arquivo ARQUIVO.pdf --tipo TIPO` inspeciona apenas o arquivo local (sem sessão nem rede). Com `--consultar-sei`, a CLI consulta o processo, escolhe o cadastro externo, recarrega o tipo (POST com sinalizador **1**, que não cadastra documento) e apresenta a prévia do limite/tipo e do PDF, sem upload. `--confirmar` envia o PDF (upload temporário) e cadastra o documento, uma única vez cada, sem retry, e só declara sucesso depois de reler a árvore e achar o externo novo (`Anexo (NNNNNNN)`, ícone PDF). Efeito não comprovado sai 1 com `Error: TentativaNaoComprovada { upload_tentado, requisicoes }` e, em `Caused by:`, o `ErroConfirmacaoExterna` da etapa (por exemplo `Sequencia(UploadIncerto(..))`, `Sequencia(CadastroIncerto(..))`, `Sequencia(ReleituraIncerta(..))`); não repita `--confirmar` sem conferir o processo. **Validado ao vivo na UnB de treinamento em 09/11/2026** (saída 0, documento `0012273`, [relatório](../../docs/validacao-live/2026-11-09-documento-incluir-externo-unb.md)). Não há chave nem trava de ensaio. Aprovada em treinamento, a escrita vale também em produção, para uso dos usuários (decisão do proprietário de 10/11/2026; `src/origem_de_treinamento.rs`).

## Contrato atual do núcleo local

- O NUP é validado pelo tipo `NumeroProcesso`. O tipo solicitado é texto opaco não vazio, sem caracteres de controle; não é declarado como existente no SEI.
- O arquivo tem nome UTF-8, sem `&#` (recusado pelo SEI observado), caracteres de controle ou marcadores direcionais Unicode (segurança da apresentação humana), e extensão `.pdf` sem distinção de caixa, é arquivo regular, não é link simbólico (inclusive ancestrais na política desta operação), não é vazio e começa com assinatura `%PDF-`. **A assinatura não comprova que o documento é um PDF estruturalmente íntegro**; o SEI continua responsável por aceitar o conteúdo.
- O SHA-256 e o tamanho são calculados lendo o arquivo em blocos, sem carregar inteiro em memória. O mesmo descritor é conferido depois da leitura: tamanho, data de modificação e, no Unix, ctime diferentes recusam a inspeção como `AlteradoDuranteLeitura`. Isso detecta alterações durante a leitura, **não garante imutabilidade futura nem resolve a troca concorrente de ancestrais por symlink**. O limite só é aplicado quando **observado no cadastro**; a inspeção da CLI ainda não inventa um teto de upload.
- Saída `--json` de sucesso: `escopo:"inspecao_local"`, `processo`, `tipo_solicitado`, `arquivo:{nome,tamanho_bytes,sha256}`, `cadastro_sei_observado:false`, `previa_remota:false`, `confirmado:false`, `requisicoes:0`, `escrita_remota:false`. O conteúdo do arquivo e credenciais jamais são publicados.
- NUP ou tipo fora do formato: erro do clap, saída 2. Arquivo recusado: saída 1, stdout vazio e `Error: <FalhaPdf::…>` em stderr. Nos dois casos, nenhum acesso à sessão ou à rede.

## Causas tipadas na inspeção local

As recusas preservam a identidade do erro Rust: o stderr traz o nome do erro
(`Error: Ausente`, `Error: AlteradoDuranteLeitura`…), sem código, mensagem
ou tabela paralela de nomes. O nome do erro é a classificação.

A família `FalhaPdf` distingue ausência, arquivo não regular, link
simbólico, arquivo vazio, extensão incorreta, nome não aceito, assinatura
incorreta, limite observado ultrapassado, alteração detectada durante o hash e falha de leitura. Arquivo não aceito sai 1.
Nenhuma delas abre sessão ou rede.

## Recibo temporário — núcleo offline da PR #380

A função Rust `documento_externo_incluir::recibo::interpretar_resposta` reconhece o recibo `nome_upload#nome#tipo#tamanho#data_hora#` observado no Todos/UnB e a recusa explícita `ERRO#motivo`. O formato tem seis campos, com o último vazio. Um recibo só é aceito quando HTTP 200, nome e tamanho conferem com o PDF esperado e não há campo ausente ou adicional; resposta divergente, HTML genérico ou status inesperado vira `FalhaUpload::EfeitoIncerto`, sem autorizar cadastro ou retry. Esta etapa é **somente um interpretador testado com respostas sintéticas**, ainda sem transporte e sem vínculo com a CLI de escrita. Um recibo válido não prova documento criado.

## Preparação multipart offline — continuação da #380

O módulo `documento_externo_incluir::multipart` monta um único corpo
`multipart/form-data` para o campo `filArquivo`, com
`Content-Disposition`, `Content-Type: application/pdf`, delimitadores
e tamanho total **calculado antes do envio** (`Content-Length`).
A função recebe o charset da tela já observada (`Iso88591` ou `Utf8`),
codifica o nome sem substituir silenciosamente caracteres e transmite os
bytes do PDF a partir de um leitor, sem materializar todo o arquivo em memória.
Nome não representável no charset, fronteira inválida e tamanho que exceda
`u64` são erros tipados `FalhaMultipart`. Os testes conferem o corpo byte a
byte, nome acentuado nos dois charsets e o cálculo do comprimento.

**Nenhum transporte remoto chama esse módulo ainda**; tampouco há uma
prévia Web validada, upload ou confirmação. O valor de `Content-Length`
não prova que os bytes de um futuro envio serão idênticos aos inspecionados:
a vinculação à mesma abertura/descritor será feita na integração com o
transporte. (Histórico desta fatia; ver o estado atual no início.)

## Transporte e recibo HTTP — bancada sintética da PR #380

O cliente HTTPS único agora aceita um POST com leitor de corpo e tamanho
conhecido. Ele usa a mesma conexão TLS/proxy dos demais comandos, envia
`Content-Length` explícito, transmite o PDF em fluxo e não repete a
requisição. O interpretador consome a resposta HTTP limitada do upload
temporário, decodifica o charset declarado e distingue recibo válido,
recusa explícita e escrita não confirmada, inclusive se o corpo for ilegível.

O cenário BDD no proxy local exercita, numa conexão única:
**Dado** PDF local e URL sintética de upload, **quando** o envio multipart
ocorre e o servidor devolve o recibo, **então** os bytes, tamanho,
`Content-Type` e recibo temporário são conferidos sem cadastro definitivo.
A URL/cookie do teste são sintéticos; a camada TLS é substituída.
**Nenhum comando da CLI chama ainda esse transporte**. Não é prova live
do SEI, não habilita `--confirmar` e não comprova documento criado.

## Escopo remoto ainda não implementado

O oráculo [Todos](https://github.com/franklinbaldo/todos/blob/main/src/todos/documento_externo.py) descreve na UnB de TREINAMENTO (11/10/2026) **duas etapas HTTP mutadoras diferentes sob uma única confirmação**: upload temporário multipart `documento_upload_anexo` → recibo → cadastro `documento_receber` com `hdnAnexos` → releitura independente da árvore e metadados do anexo. Não é correto exigir apenas um POST total, nem interpretar recibo de upload como documento já incluído.

A próxima fatia deve observar cada campo e action servidos; conferir os tipos e limites; usar `formulario_web`, origem e identidade; executar um único upload e um único cadastro, **sem retry**; distinguir `anexo confirmado`, `anexo não observado`, `documento criado` e `escrita não confirmada` com erros tipados, preferindo saída 1 a sucesso não provado. O caso [Todos #1246](https://github.com/franklinbaldo/todos/issues/1246) relata anexo somente visível na tela de alteração, não na árvore; **não** autoriza inferir ausência nem repetir POST.

**Evidência:** os testes sintéticos do núcleo estão em `src/documento_externo_incluir.rs`; live em treinamento: PASS em 09/11/2026 (ver o início deste contrato).

## Plano da linha de anexo pela tela servida — fatia #391

O núcleo Rust \`documento_externo_incluir::anexo::ler_plano\` lê declarações literais de **script inline** do cadastro observado: \`new infraUpload('frmAnexos', URL)\` (SEI 4) ou com terceiro argumento booleano (SEI 5), e a ordem de \`objTabelaAnexos.adicionar([...])\`. Elementos em \`template\`, scripts externos, chamadas ausentes, repetidas e campos não reconhecidos não autorizam a derivação. Nenhum JavaScript é executado e não se constrói URL de upload. O destino **relativo publicado** fica privado no plano e só poderá ser consumido por transporte depois da validação de origem/sessão/processo.

Dado um \`ReciboUpload\` aceito pelo interpretador já existente, o plano monta a **linha única** para \`hdnAnexos\` com a ordem servida, literais da tela e o tamanho apresentado pela função observada \`infraFormatarTamanhoBytes\` (arredondamento de duas casas). O separador \`±\` não pode atravessar um valor de recibo nem alterar o número de colunas. Esse campo é apenas a representação para um futuro cadastro; não constitui prova de documento criado.

**BDD offline:** UnB SEI 4, PEN3 SEI 5 (terceiro argumento booleano), reordenação de colunas, coluna desconhecida, múltiplos uploads, \`template\` e recibo com separador. **A implementação continua WIP:** o núcleo não está conectado à CLI, não transmite os dois POSTs e não habilita \`--confirmar\`. Ainda faltam prévia Web com tipo/limites, cadastro definitivo, releitura independente e LIVE em treinamento. Testes sintéticos não são evidência live.

## Cadastro recarregado e limite de PDF — fatia #394

O núcleo `documento_externo_incluir::cadastro::observar_cadastro` interpreta o formulário `frmDocumentoCadastro` **já recarregado** depois da escolha de `selSerie`. Confere a opção do tipo solicitado exatamente uma vez, selecionada e coerente com `hdnIdSerie`; exige os controles ocultos de sinalizador e anexos. Lê a extensão `pdf` e o teto em MiB da declaração literal `arrExt` publicada no script servido, sem executar JavaScript; o arquivo acima desse teto é recusado antes de eventual upload. Em seguida, deriva o plano `hdnAnexos` já implementado, com o destino assinado mantido privado.

**BDD sintético**: cadastro UnB SEI 4, terceiro argumento de upload no PEN3 SEI 5, tipo não selecionado, ausente ou duplicado, PDF acima do limite e extensão PDF não oferecida. São casos do acervo do Todos, não comprovação live. **CLI ainda não ligada a essa leitura:** `ies documento incluir-externo` permanece na inspeção local, e `--confirmar` continua recusando antes da sessão/rede. Faltam prévia Web integrada, confirmação única das duas escritas e releitura independente; LIVE: NOT_RUN.

## Prévia Web somente leitura — continuação #394

**Dado** um processo aberto na unidade e o link de inclusão publicado na barra, **quando** executo `ies documento incluir-externo --processo NUP --arquivo PDF --tipo Anexo --consultar-sei`, **então** a CLI abre a lista, escolhe a alternativa `Externo` publicada pelo SEI 4 como link `documento_receber` ou pelo SEI 5 via `escolher(-1)`, lê o cadastro inicial, identifica `selSerie` e o recarrega com `hdnFlagDocumentoCadastro=1`. O retorno deve conter o mesmo tipo selecionado, limite de `pdf` e plano de upload observado, sem transmitir o arquivo. O POST de recarga é navegação de formulário, **não** cadastro definitivo. Sinalizador `2` na tela inicial recusa antes de enviar qualquer POST.

A saída `--json` acrescenta `escopo:"previa_web"`, `previa_remota:true`, `cadastro_sei_observado:true`, `id_tipo`, `unidade`, `limite_pdf_bytes`, `campos_oferecidos` (nomes dos controles visíveis/habilitados presentes no formulário, sem os valores), `arquivo` (nome/tamanho/SHA-256), `confirmado:false`, `escrita_remota:false` e contagem de requisições. Falha remota da prévia: saída 1 e `Error: <ErroPreviaExterna::…>` (a etapa: `LinkNaoPublicado`, `TipoNaoOferecido`, `CadastroNaoConferido`…); status HTTP e mensagem saneada vêm em `Transporte(Status(StatusHttpRecusado { status, mensagem_do_servidor }))`. A contagem de requisições não acompanha a falha da prévia. A inspeção local padrão conserva exatamente sua saída anterior.

BDD sintético com transporte roteirizado (Todos UnB SEI 4): abertura → GET da lista → GET do cadastro inicial → um POST de **recarga**, sem `documento_upload_anexo` e sem POST definitivo; cenário com flag `2` recusa sem POST. Também há BDD de parsing da alternativa SEI 5 e do tipo inicial. O ensaio ao vivo do IES em instalação de treinamento ainda é **NOT_RUN**: os testes e replays sintéticos não provam disponibilidade remota de todas as instalações. **Confirmar ainda não está implementado**; faltam upload temporário único, cadastro definitivo e releitura independente, sem retry.
