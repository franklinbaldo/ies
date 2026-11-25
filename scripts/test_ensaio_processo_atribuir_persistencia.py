"""Ensaios de persistência do checkpoint da atribuição, sem rede."""
import json
import unittest

from ensaio_processo_atribuir import ensaiar
from test_ensaio_processo_atribuir import BaseEnsaio, NUP, ORIGEM, USUARIO, roteiro


class EnsaioPersistencia(BaseEnsaio, unittest.TestCase):
    def test_trava_sincroniza_diretorio_antes_do_primeiro_post(self):
        import os
        import stat
        from unittest.mock import patch

        fsync_original = os.fsync
        diretorio_sincronizado = False
        respostas = iter(roteiro())

        def fsync_rastreado(fd):
            nonlocal diretorio_sincronizado
            if stat.S_ISDIR(os.fstat(fd).st_mode):
                diretorio_sincronizado = True
            return fsync_original(fd)

        def executar(_ies, argumentos):
            self.chamadas.append(argumentos)
            if "--confirmar" in argumentos:
                self.assertTrue(diretorio_sincronizado)
            return next(respostas)

        with patch("ensaio_processo_atribuir.os.fsync", side_effect=fsync_rastreado):
            codigo, relatorio = ensaiar(
                ies="ies-falso", registro=self.registro, nup=NUP,
                usuario=USUARIO, origem=ORIGEM, executar=executar,
                pausa=lambda _: None,
            )
        self.assertEqual((codigo, relatorio["estado"]), (0, "PASS"))
        self.assertEqual(len(self.chamadas), 6)

    def test_falha_fsync_diretorio_bloqueia_primeiro_post(self):
        import os
        import stat
        from unittest.mock import patch

        fsync_original = os.fsync

        def fsync_falhando_no_diretorio(fd):
            if stat.S_ISDIR(os.fstat(fd).st_mode):
                raise OSError("diretorio nao persistido")
            return fsync_original(fd)

        with patch("ensaio_processo_atribuir.os.fsync",
                   side_effect=fsync_falhando_no_diretorio):
            codigo, relatorio = self.rodar()
        self.assertEqual((codigo, relatorio["estado"]), (2, "BLOQUEADO"))
        self.assertEqual(len(self.chamadas), 1)
        self.assertTrue(self.registro.exists())

    def test_rename_final_sincroniza_diretorio_antes_de_pass(self):
        import os
        import stat
        from unittest.mock import patch

        fsync_original = os.fsync
        replace_original = os.replace
        houve_rename = False
        diretorio_apos_rename = False

        def substituir(*args):
            nonlocal houve_rename
            replace_original(*args)
            houve_rename = True

        def sincronizar(fd):
            nonlocal diretorio_apos_rename
            if stat.S_ISDIR(os.fstat(fd).st_mode) and houve_rename:
                diretorio_apos_rename = True
            return fsync_original(fd)

        with patch("ensaio_processo_atribuir.os.replace", side_effect=substituir), \
             patch("ensaio_processo_atribuir.os.fsync", side_effect=sincronizar):
            codigo, relatorio = self.rodar()
        self.assertEqual((codigo, relatorio["estado"]), (0, "PASS"))
        self.assertTrue(houve_rename)
        self.assertTrue(diretorio_apos_rename)

    def test_falha_fsync_diretorio_apos_rename_nao_declara_pass(self):
        import os
        import stat
        from unittest.mock import patch

        fsync_original = os.fsync
        chamadas_diretorio = 0

        def sincronizar(fd):
            nonlocal chamadas_diretorio
            if stat.S_ISDIR(os.fstat(fd).st_mode):
                chamadas_diretorio += 1
                if chamadas_diretorio == 2:
                    raise OSError("diretorio nao persistido apos rename")
            return fsync_original(fd)

        with patch("ensaio_processo_atribuir.os.fsync", side_effect=sincronizar):
            codigo, relatorio = self.rodar()
        self.assertEqual(chamadas_diretorio, 2)
        self.assertEqual((codigo, relatorio["estado"]), (3, "EFEITO_INCERTO"))
        self.assertEqual(relatorio["residuo"], "possivel_remocao")
        self.assertEqual(len(self.chamadas), 6)

    def test_escrita_parcial_do_checkpoint_antes_do_post_eh_completa(self):
        import os
        from unittest.mock import patch

        escrita_original = os.write
        parcial_injetada = False
        registro_antes_do_post = []
        respostas = iter(roteiro())

        def escrever_parcial(fd, dados):
            nonlocal parcial_injetada
            if not parcial_injetada:
                parcial_injetada = True
                return escrita_original(fd, dados[:1])
            return escrita_original(fd, dados)

        def executar(_ies, args):
            if "--confirmar" in args and not registro_antes_do_post:
                registro_antes_do_post.append(self.registro.read_text())
            return next(respostas)

        with patch("ensaio_processo_atribuir.os.write", side_effect=escrever_parcial):
            codigo, relatorio = ensaiar(
                ies="ies-falso", registro=self.registro, nup=NUP,
                usuario=USUARIO, origem=ORIGEM, executar=executar,
                pausa=lambda _: None,
            )
        self.assertTrue(parcial_injetada)
        self.assertEqual((codigo, relatorio["estado"]), (0, "PASS"))
        self.assertEqual(json.loads(registro_antes_do_post[0])["estado"],
                         "CONFIRMACAO_INICIADA")
        self.assertEqual(json.loads(self.registro.read_text())["estado"], "PASS")
