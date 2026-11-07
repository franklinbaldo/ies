---
type: CommandContract
title: Inclusão de documento externo PDF — inspeção local antes da escrita
operation: documento.incluir-externo
command: ies documento incluir-externo --processo NUP --arquivo ARQUIVO.pdf --tipo TIPO [--confirmar]
effect: write-document-external
implementation: partial
---

# Incluir documento externo em processo — #332

A capacidade está **PARCIAL LOCAL**: `ies documento incluir-externo --processo NUP --arquivo ARQUIVO.pdf --tipo TIPO` inspeciona **somente o arquivo local**, não a disponibilidade do formulário remoto. É uma etapa de preparação, **não uma prévia do SEI**. Não abre sessão, não consulta o processo, não oferece lista de tipos e não transmite o arquivo. A flag `--confirmar` é reconhecida, mas **recusa antes de sessão e rede** (causa tipada `CadastroNaoValidado`) até comprovar a operação completa.

## Contrato atual do núcleo local

- O NUP é validado pelo tipo `NumeroProcesso`. O tipo solicitado é texto opaco não vazio, sem caracteres de controle; não é declarado como existente no SEI.
- O arquivo tem nome UTF-8, sem `&#` (recusado pelo SEI observado), caracteres de controle ou marcadores direcionais Unicode (segurança da apresentação humana), e extensão `.pdf` sem distinção de caixa, é arquivo regular, não é link simbólico (inclusive ancestrais na política desta operação), não é vazio e começa com assinatura `%PDF-`. **A assinatura não comprova que o documento é um PDF estruturalmente íntegro**; o SEI continua responsável por aceitar o conteúdo.
- O SHA-256 e o tamanho são calculados lendo o arquivo em blocos, sem carregar inteiro em memória. O mesmo descritor é conferido depois da leitura: tamanho, data de modificação e, no Unix, ctime diferentes recusam a inspeção como `AlteradoDuranteLeitura`. Isso detecta alterações durante a leitura, **não garante imutabilidade futura nem resolve a troca concorrente de ancestrais por symlink**. O limite só é aplicado quando **observado no cadastro**; a inspeção da CLI ainda não inventa um teto de upload.
- Saída `--json` de sucesso: `escopo:"inspecao_local"`, `processo`, `tipo_solicitado`, `arquivo:{nome,tamanho_bytes,sha256}`, `cadastro_sei_observado:false`, `previa_remota:false`, `confirmado:false`, `requisicoes:0`, `escrita_remota:false`. O conteúdo do arquivo e credenciais jamais são publicados.
- Qualquer falha de entrada: saída 2, `causa` serializada diretamente pelo enum, `retentavel:false`, `escrita_remota:false` e nenhum acesso à sessão/rede.

## Causas tipadas na inspeção local

As recusas preservam a identidade do erro Rust. A saída JSON publica a
variante diretamente via Serde no campo `causa`, sem `subcodigo` nem tabela
paralela de nomes: por exemplo, `{"causa":{"Pdf":"NomeInvalido"}}`,
`{"causa":{"Tipo":"Vazio"}}`, `{"causa":{"Processo":"Controle"}}` e
`{"causa":"CadastroNaoValidado"}`. A mensagem humana é somente
apresentação, nunca critério de classificação.

A família `FalhaPdf` distingue ausência, arquivo não regular, link
simbólico, arquivo vazio, extensão incorreta, nome não aceito, assinatura
incorreta, limite observado ultrapassado, alteração detectada durante o hash e falha de leitura. A recusa
de `--confirmar` ainda não validado sai 1; entrada não aceita sai 2.
Nenhuma delas abre sessão ou rede.

## Escopo remoto ainda não implementado

O oráculo [Todos](https://github.com/franklinbaldo/todos/blob/main/src/todos/documento_externo.py) descreve na UnB de TREINAMENTO (11/10/2026) **duas etapas HTTP mutadoras diferentes sob uma única confirmação**: upload temporário multipart `documento_upload_anexo` → recibo → cadastro `documento_receber` com `hdnAnexos` → releitura independente da árvore e metadados do anexo. Não é correto exigir apenas um POST total, nem interpretar recibo de upload como documento já incluído.

A próxima fatia deve observar cada campo e action servidos; conferir os tipos e limites; usar `formulario_web`, origem e identidade; executar um único upload e um único cadastro, **sem retry**; distinguir `anexo confirmado`, `anexo não observado`, `documento criado` e `efeito incerto` com variantes tipadas, preferindo saída 3 a sucesso não provado. O caso [Todos #1246](https://github.com/franklinbaldo/todos/issues/1246) relata anexo somente visível na tela de alteração, não na árvore; **não** autoriza inferir ausência nem repetir POST.

**Evidência:** os testes sintéticos do núcleo estão em `src/documento_externo_incluir.rs`; live em treinamento: **NOT_RUN**. `--confirmar` permanece fechado.
